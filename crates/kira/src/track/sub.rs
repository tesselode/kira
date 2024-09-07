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
	command::{CommandReader, ValueChangeCommand},
	effect::Effect,
	info::{Info, ListenerInfo},
	listener::ListenerId,
	manager::backend::resources::{
		clocks::Clocks, listeners::Listeners, modulators::Modulators, ResourceStorage,
	},
	sound::Sound,
	tween::{Easing, Parameter, Tweenable},
	Frame, Volume,
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
	volume: Parameter<Volume>,
	set_volume_command_reader: CommandReader<ValueChangeCommand<Volume>>,
	sounds: ResourceStorage<Box<dyn Sound>>,
	sub_tracks: ResourceStorage<Track>,
	effects: Vec<Box<dyn Effect>>,
	sends: Vec<(SendTrackId, SendTrackRoute)>,
	persist_until_sounds_finish: bool,
	spatial_data: Option<SpatialData>,
}

impl Track {
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
		self.volume
			.read_command(&mut self.set_volume_command_reader);
		for (_, route) in &mut self.sends {
			route.read_commands();
		}
		if let Some(SpatialData {
			position,
			set_position_command_reader,
			..
		}) = &mut self.spatial_data
		{
			position.read_command(set_position_command_reader);
		}
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
		dt: f64,
		clocks: &Clocks,
		modulators: &Modulators,
		listeners: &Listeners,
		spatial_track_position: Option<Vec3>,
		send_tracks: &mut ResourceStorage<SendTrack>,
	) -> Frame {
		let spatial_track_position = self
			.spatial_data
			.as_ref()
			.map(|spatial_data| spatial_data.position.value())
			.or(spatial_track_position);
		let info = Info::new(
			&clocks.0.resources,
			&modulators.0.resources,
			&listeners.0.resources,
			spatial_track_position,
		);
		self.volume.update(dt, &info);
		for (_, route) in &mut self.sends {
			route.volume.update(dt, &info);
		}

		let mut output = Frame::ZERO;
		for (_, sub_track) in &mut self.sub_tracks {
			output += sub_track.process(
				dt,
				clocks,
				modulators,
				listeners,
				spatial_track_position,
				send_tracks,
			);
		}
		for (_, sound) in &mut self.sounds {
			output += sound.process(dt, &info);
		}
		if let Some(spatial_data) = &mut self.spatial_data {
			spatial_data.position.update(dt, &info);
			if let Some(ListenerInfo {
				position,
				orientation,
			}) = info.listener_info(spatial_data.listener_id)
			{
				output = spatial_data.spatialize(output, position.into(), orientation.into());
			}
		}
		for effect in &mut self.effects {
			output = effect.process(output, dt, &info);
		}
		for (send_track_id, SendTrackRoute { volume, .. }) in &self.sends {
			let Some(send_track) = send_tracks.get_mut(send_track_id.0) else {
				continue;
			};
			send_track.add_input(output * volume.value().as_amplitude() as f32);
		}
		output *= self.volume.value().as_amplitude() as f32;
		output
	}
}

struct SpatialData {
	listener_id: ListenerId,
	position: Parameter<Vec3>,
	set_position_command_reader: CommandReader<ValueChangeCommand<Vec3>>,
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
				Volume::Decibels(Volume::MIN_DECIBELS),
				Volume::Decibels(0.0),
				relative_volume.into(),
			)
			.as_amplitude() as f32;
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
}
