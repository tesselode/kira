mod builder;
mod handle;

pub use builder::*;
pub use handle::*;

use std::{error::Error, fmt::Display, sync::Arc};

use glam::Vec3;

use crate::{
	command::ValueChangeCommand,
	command_writers_and_readers,
	effect::Effect,
	info::Info,
	playback_state_manager::PlaybackStateManager,
	// listener::ListenerId,
	resources::{
		clocks::Clocks, /* listeners::Listeners, */ modulators::Modulators, ResourceStorage,
	},
	sound::Sound,
	tween::{Parameter, Tween},
	Decibels,
	Frame,
	StartTime,
	INTERNAL_BUFFER_SIZE,
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
	// spatial_data: Option<SpatialData>,
	playback_state_manager: PlaybackStateManager,
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
			effect.init(sample_rate);
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

	pub fn process(
		&mut self,
		out: &mut [Frame],
		dt: f64,
		clocks: &Clocks,
		modulators: &Modulators,
		// listeners: &Listeners,
		// parent_spatial_track_info: Option<SpatialTrackInfo>,
		send_tracks: &mut ResourceStorage<SendTrack>,
	) {
		let info = Info::new(
			&clocks.0.resources,
			&modulators.0.resources,
			// &listeners.0.resources,
			// spatial_track_info,
		);

		for (i, frame) in out.iter_mut().enumerate() {
			let changed_playback_state = self
				.playback_state_manager
				.update(dt, &info.for_single_frame(i));
			if changed_playback_state {
				self.update_shared_playback_state();
			}
			if !self.playback_state_manager.playback_state().is_advancing() {
				*frame = Frame::ZERO;
				continue;
			}

			let mut single_frame_out = [Frame::ZERO];

			// process sub-tracks
			for (_, sub_track) in &mut self.sub_tracks {
				let mut sub_track_out = [Frame::ZERO];
				sub_track.process(&mut sub_track_out, dt, clocks, modulators, send_tracks);
				single_frame_out[0] += sub_track_out[0];
			}

			// process sounds
			for (_, sound) in &mut self.sounds {
				let mut sound_out = [Frame::ZERO];
				sound.process(&mut sound_out, dt, &info);
				single_frame_out[0] += sound_out[0];
			}

			// process effects
			for effect in &mut self.effects {
				effect.process(&mut single_frame_out, dt, &info);
			}

			*frame = single_frame_out[0] * self.playback_state_manager.fade_volume().as_amplitude();
		}

		/* let spatial_track_info = self
		.spatial_data
		.as_ref()
		.map(|spatial_data| SpatialTrackInfo {
			position: spatial_data.position.value(),
			listener_id: spatial_data.listener_id,
		})
		.or(parent_spatial_track_info); */

		/* if let Some(spatial_data) = &mut self.spatial_data {
			spatial_data.position.update(dt, &info);
			if let Some(ListenerInfo {
				position,
				orientation,
			}) = info.listener_info()
			{
				output = spatial_data.spatialize(output, position.into(), orientation.into());
			}
		} */

		// apply post-effects volume
		let mut volume_buffer = [Decibels::IDENTITY; INTERNAL_BUFFER_SIZE];
		self.volume
			.update_chunk(&mut volume_buffer[..out.len()], dt, &info);
		for (frame, volume) in out.iter_mut().zip(volume_buffer.iter().copied()) {
			*frame *= volume.as_amplitude();
		}

		for (send_track_id, SendTrackRoute { volume, .. }) in &mut self.sends {
			let Some(send_track) = send_tracks.get_mut(send_track_id.0) else {
				continue;
			};
			for (i, frame) in out.iter().copied().enumerate() {
				volume.update(dt, &info.for_single_frame(i));
				send_track.add_input(i, frame * volume.value().as_amplitude());
			}
		}
	}

	fn read_commands(&mut self) {
		self.volume
			.read_command(&mut self.command_readers.set_volume);
		for (_, route) in &mut self.sends {
			route.read_commands();
		}
		/* if let Some(SpatialData { position, .. }) = &mut self.spatial_data {
			position.read_command(&mut self.command_readers.set_position);
		} */
		if let Some(tween) = self.command_readers.pause.read() {
			self.pause(tween);
		}
		if let Some((start_time, tween)) = self.command_readers.resume.read() {
			self.resume(start_time, tween);
		}
	}
}

/* struct SpatialData {
	listener_id: ListenerId,
	position: Parameter<Vec3>,
	/// The distances from a listener at which the track is loudest and quietest.
	distances: SpatialTrackDistances,
	/// How the track's volume will change with distance.
	///
	/// If `None`, the track will output at a constant volume.
	attenuation_function: Option<Easing>,
	/// Whether the track's output should be panned left or right depending on its
	/// direction from the listener.
	enable_spatialization: bool,
}

impl SpatialData {
	fn spatialize(
		&self,
		input: Frame,
		listener_position: Vec3,
		listener_orientation: Quat,
	) -> Frame {
		const MIN_EAR_AMPLITUDE: f32 = 0.5;

		let mut output = input;
		// attenuate volume
		if let Some(attenuation_function) = self.attenuation_function {
			let distance = (listener_position - self.position.value()).length();
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
		if self.enable_spatialization {
			output = output.as_mono();
			let (left_ear_position, right_ear_position) =
				listener_ear_positions(listener_position, listener_orientation);
			let (left_ear_direction, right_ear_direction) =
				listener_ear_directions(listener_orientation);
			let emitter_direction_relative_to_left_ear =
				(self.position.value() - left_ear_position).normalize_or_zero();
			let emitter_direction_relative_to_right_ear =
				(self.position.value() - right_ear_position).normalize_or_zero();
			let left_ear_volume =
				(left_ear_direction.dot(emitter_direction_relative_to_left_ear) + 1.0) / 2.0;
			let right_ear_volume =
				(right_ear_direction.dot(emitter_direction_relative_to_right_ear) + 1.0) / 2.0;
			output.left *= MIN_EAR_AMPLITUDE + (1.0 - MIN_EAR_AMPLITUDE) * left_ear_volume;
			output.right *= MIN_EAR_AMPLITUDE + (1.0 - MIN_EAR_AMPLITUDE) * right_ear_volume;
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
} */

command_writers_and_readers! {
	set_volume: ValueChangeCommand<Decibels>,
	set_position: ValueChangeCommand<Vec3>,
	pause: Tween,
	resume: (StartTime, Tween),
}
