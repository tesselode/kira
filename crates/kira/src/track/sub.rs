mod builder;
mod handle;
mod spatial_builder;
mod spatial_handle;

pub use builder::*;
pub use handle::*;
pub use spatial_builder::*;
pub use spatial_handle::*;

use std::{error::Error, f32::consts::FRAC_PI_8, fmt::Display, sync::Arc};

use glam::{Quat, Vec3};

use crate::{
	backend::resources::{
		clocks::Clocks, listeners::Listeners, modulators::Modulators, ResourceStorage,
	},
	command::ValueChangeCommand,
	command_writers_and_readers,
	effect::Effect,
	info::{Info, SpatialTrackInfo},
	listener::ListenerId,
	playback_state_manager::PlaybackStateManager,
	sound::Sound,
	Decibels, Easing, Frame, Parameter, StartTime, Tween, Tweenable,
};

use super::{SendTrack, SendTrackId, SendTrackRoute, TrackShared};

/// An error that's returned when trying to change the volume of a track route
/// that did not exist originally.
#[derive(Debug)]
pub struct NonexistentRoute;

impl Display for NonexistentRoute {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str("Cannot change the volume of a track route that did not exist originally")
	}
}

impl Error for NonexistentRoute {}

pub(crate) struct Track {
	shared: Arc<TrackShared>,
	command_readers: CommandReaders,
	volume: Parameter<Decibels>,
	sounds: ResourceStorage<Box<dyn Sound>>,
	sub_tracks: ResourceStorage<Track>,
	effects: Vec<Box<dyn Effect>>,
	sends: Vec<(SendTrackId, SendTrackRoute)>,
	persist_until_sounds_finish: bool,
	spatial_data: Option<SpatialData>,
	playback_state_manager: PlaybackStateManager,
	temp_buffer: Vec<Frame>,
	internal_buffer_size: usize,
}

impl Track {
	fn update_shared_playback_state(&mut self) {
		self.shared
			.set_state(self.playback_state_manager.playback_state());
	}

	fn pause(&mut self, fade_out_tween: Tween) {
		self.playback_state_manager.pause(fade_out_tween);
		self.update_shared_playback_state();
	}

	fn resume(&mut self, start_time: StartTime, fade_in_tween: Tween) {
		self.playback_state_manager
			.resume(start_time, fade_in_tween);
		self.update_shared_playback_state();
	}

	pub fn init_effects(&mut self, sample_rate: u32) {
		for effect in &mut self.effects {
			effect.init(sample_rate, self.internal_buffer_size);
		}
		for (_, sub_track) in &mut self.sub_tracks {
			sub_track.init_effects(sample_rate);
		}
	}

	pub fn on_change_sample_rate(&mut self, sample_rate: u32) {
		for effect in &mut self.effects {
			effect.on_change_sample_rate(sample_rate);
		}
		for (_, sub_track) in &mut self.sub_tracks {
			sub_track.on_change_sample_rate(sample_rate);
		}
	}

	#[must_use]
	pub fn shared(&self) -> Arc<TrackShared> {
		self.shared.clone()
	}

	pub fn should_be_removed(&self) -> bool {
		if self
			.sub_tracks
			.iter()
			.any(|(_, sub_track)| !sub_track.should_be_removed())
		{
			return false;
		}
		if self.persist_until_sounds_finish {
			self.shared().is_marked_for_removal() && self.sounds.is_empty()
		} else {
			self.shared().is_marked_for_removal()
		}
	}

	pub fn on_start_processing(&mut self) {
		self.read_commands();
		self.sounds.remove_and_add(|sound| sound.finished());
		for (_, sound) in &mut self.sounds {
			sound.on_start_processing();
		}
		self.sub_tracks
			.remove_and_add(|sub_track| sub_track.should_be_removed());
		for (_, sub_track) in &mut self.sub_tracks {
			sub_track.on_start_processing();
		}
		for effect in &mut self.effects {
			effect.on_start_processing();
		}
	}

	#[allow(clippy::too_many_arguments)]
	pub fn process(
		&mut self,
		out: &mut [Frame],
		dt: f64,
		clocks: &Clocks,
		modulators: &Modulators,
		listeners: &Listeners,
		parent_spatial_track_info: Option<SpatialTrackInfo>,
		send_tracks: &mut ResourceStorage<SendTrack>,
	) {
		let spatial_track_info = self
			.spatial_data
			.as_ref()
			.map(|spatial_data| SpatialTrackInfo {
				position: spatial_data.position.value(),
				listener_id: spatial_data.listener_id,
			})
			.or(parent_spatial_track_info);
		let info = Info::new(
			&clocks.0.resources,
			&modulators.0.resources,
			&listeners.0.resources,
			spatial_track_info,
		);
		self.volume.update(dt * out.len() as f64, &info);
		for (_, route) in &mut self.sends {
			route.volume.update(dt * out.len() as f64, &info);
		}

		let changed_playback_state = self
			.playback_state_manager
			.update(dt * out.len() as f64, &info);
		if changed_playback_state {
			self.update_shared_playback_state();
		}
		if !self.playback_state_manager.playback_state().is_advancing() {
			out.fill(Frame::ZERO);
			return;
		}

		let num_frames = out.len();
		for (_, sub_track) in &mut self.sub_tracks {
			sub_track.process(
				&mut self.temp_buffer[..out.len()],
				dt,
				clocks,
				modulators,
				listeners,
				spatial_track_info,
				send_tracks,
			);
			for (summed_out, track_out) in out.iter_mut().zip(self.temp_buffer.iter().copied()) {
				*summed_out += track_out;
			}
			self.temp_buffer.fill(Frame::ZERO);
		}
		for (_, sound) in &mut self.sounds {
			sound.process(&mut self.temp_buffer[..out.len()], dt, &info);
			for (summed_out, sound_out) in out.iter_mut().zip(self.temp_buffer.iter().copied()) {
				*summed_out += sound_out;
			}
			self.temp_buffer.fill(Frame::ZERO);
		}
		for effect in &mut self.effects {
			effect.process(out, dt, &info);
		}
		if let Some(spatial_data) = &mut self.spatial_data {
			spatial_data.position.update(dt * out.len() as f64, &info);
			spatial_data
				.spatialization_strength
				.update(dt * out.len() as f64, &info);
			for (i, frame) in out.iter_mut().enumerate() {
				let time_in_chunk = i as f64 / num_frames as f64;
				if let Some(listener_info) = info.listener_info() {
					let interpolated_position =
						listener_info.interpolated_position(time_in_chunk as f32);
					let interpolated_orientation =
						listener_info.interpolated_orientation(time_in_chunk as f32);
					*frame = spatial_data.spatialize(
						*frame,
						interpolated_position.into(),
						interpolated_orientation.into(),
						time_in_chunk,
					);
				}
			}
		}
		for (i, frame) in out.iter_mut().enumerate() {
			let time_in_chunk = (i + 1) as f64 / num_frames as f64;
			let volume = self.volume.interpolated_value(time_in_chunk).as_amplitude();
			let fade_volume = self
				.playback_state_manager
				.interpolated_fade_volume(time_in_chunk)
				.as_amplitude();
			*frame *= volume * fade_volume;
		}
		for (send_track_id, SendTrackRoute { volume, .. }) in &self.sends {
			let Some(send_track) = send_tracks.get_mut(send_track_id.0) else {
				continue;
			};
			send_track.add_input(out, volume.value());
		}
	}

	fn read_commands(&mut self) {
		self.volume
			.read_command(&mut self.command_readers.set_volume);
		for (_, route) in &mut self.sends {
			route.read_commands();
		}
		if let Some(SpatialData {
			position,
			spatialization_strength,
			..
		}) = &mut self.spatial_data
		{
			position.read_command(&mut self.command_readers.set_position);
			spatialization_strength
				.read_command(&mut self.command_readers.set_spatialization_strength);
		}
		if let Some(tween) = self.command_readers.pause.read() {
			self.pause(tween);
		}
		if let Some((start_time, tween)) = self.command_readers.resume.read() {
			self.resume(start_time, tween);
		}
	}
}

struct SpatialData {
	listener_id: ListenerId,
	position: Parameter<Vec3>,
	/// The distances from a listener at which the track is loudest and quietest.
	distances: SpatialTrackDistances,
	/// How the track's volume will change with distance.
	///
	/// If `None`, the track will output at a constant volume.
	attenuation_function: Option<Easing>,
	/// How much the track's output should be panned left or right depending on its
	/// direction from the listener.
	///
	/// This value should be between `0.0` and `1.0`. `0.0` disables spatialization
	/// entirely.
	spatialization_strength: Parameter<f32>,
}

impl SpatialData {
	fn spatialize(
		&self,
		input: Frame,
		listener_position: Vec3,
		listener_orientation: Quat,
		time_in_chunk: f64,
	) -> Frame {
		let position = self.position.interpolated_value(time_in_chunk);
		let spatialization_strength = self
			.spatialization_strength
			.interpolated_value(time_in_chunk)
			.clamp(0.0, 1.0);
		let min_ear_amplitude = 1.0 - spatialization_strength;

		let mut output = input;
		// attenuate volume
		if let Some(attenuation_function) = self.attenuation_function {
			let distance = (listener_position - position).length();
			let relative_distance = self.distances.relative_distance(distance);
			let relative_volume =
				attenuation_function.apply((1.0 - relative_distance).into()) as f32;
			let amplitude = Tweenable::interpolate(
				Decibels::SILENCE,
				Decibels::IDENTITY,
				relative_volume.into(),
			)
			.as_amplitude();
			output *= amplitude;
		}
		// apply spatialization
		if spatialization_strength != 0.0 {
			output = output.as_mono();
			let (left_ear_position, right_ear_position) =
				listener_ear_positions(listener_position, listener_orientation);
			let (left_ear_direction, right_ear_direction) =
				listener_ear_directions(listener_orientation);
			let emitter_direction_relative_to_left_ear =
				(position - left_ear_position).normalize_or_zero();
			let emitter_direction_relative_to_right_ear =
				(position - right_ear_position).normalize_or_zero();
			let left_ear_volume =
				(left_ear_direction.dot(emitter_direction_relative_to_left_ear) + 1.0) / 2.0;
			let right_ear_volume =
				(right_ear_direction.dot(emitter_direction_relative_to_right_ear) + 1.0) / 2.0;
			output.left *= min_ear_amplitude + (1.0 - min_ear_amplitude) * left_ear_volume;
			output.right *= min_ear_amplitude + (1.0 - min_ear_amplitude) * right_ear_volume;
		}
		output
	}
}

#[must_use]
fn listener_ear_positions(listener_position: Vec3, listener_orientation: Quat) -> (Vec3, Vec3) {
	const EAR_DISTANCE: f32 = 0.1;
	let position = listener_position;
	let orientation = listener_orientation;
	let left = position + orientation * (Vec3::NEG_X * EAR_DISTANCE);
	let right = position + orientation * (Vec3::X * EAR_DISTANCE);
	(left, right)
}

#[must_use]
fn listener_ear_directions(listener_orientation: Quat) -> (Vec3, Vec3) {
	const EAR_ANGLE_FROM_HEAD: f32 = FRAC_PI_8;
	let left_ear_direction_relative_to_head =
		Quat::from_rotation_y(-EAR_ANGLE_FROM_HEAD) * Vec3::NEG_X;
	let right_ear_direction_relative_to_head = Quat::from_rotation_y(EAR_ANGLE_FROM_HEAD) * Vec3::X;
	let orientation = listener_orientation;
	let left = orientation * left_ear_direction_relative_to_head;
	let right = orientation * right_ear_direction_relative_to_head;
	(left, right)
}

command_writers_and_readers! {
	set_volume: ValueChangeCommand<Decibels>,
	set_position: ValueChangeCommand<Vec3>,
	set_spatialization_strength: ValueChangeCommand<f32>,
	pause: Tween,
	resume: (StartTime, Tween),
}
