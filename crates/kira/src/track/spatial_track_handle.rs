use std::{
	collections::HashMap,
	sync::{atomic::Ordering, Arc},
};

use glam::Vec3;

use crate::{
	command::{CommandWriter, ValueChangeCommand},
	listener::ListenerId,
	manager::backend::{resources::ResourceController, RendererShared},
	sound::{Sound, SoundData},
	tween::{Tween, Value},
	PlaySoundError, ResourceLimitReached, Volume,
};

use super::{
	NonexistentRoute, SendTrackId, SpatialTrackBuilder, Track, TrackBuilder, TrackHandle,
	TrackShared,
};

/// Controls a mixer track.
///
/// When a [`MainTrackHandle`] is dropped, the corresponding mixer
/// track will be removed.
#[derive(Debug)]
pub struct SpatialTrackHandle {
	pub(crate) renderer_shared: Arc<RendererShared>,
	pub(crate) shared: Option<Arc<TrackShared>>,
	pub(crate) set_volume_command_writer: CommandWriter<ValueChangeCommand<Volume>>,
	pub(crate) sound_controller: ResourceController<Box<dyn Sound>>,
	pub(crate) sub_track_controller: ResourceController<Track>,
	pub(crate) send_volume_command_writers:
		HashMap<SendTrackId, CommandWriter<ValueChangeCommand<Volume>>>,
	pub(crate) set_position_command_writer: CommandWriter<ValueChangeCommand<Vec3>>,
}

impl SpatialTrackHandle {
	/// Plays a sound.
	pub fn play<D: SoundData>(
		&mut self,
		sound_data: D,
	) -> Result<D::Handle, PlaySoundError<D::Error>> {
		let (sound, handle) = sound_data
			.into_sound()
			.map_err(PlaySoundError::IntoSoundError)?;
		self.sound_controller
			.insert(sound)
			.map_err(|_| PlaySoundError::SoundLimitReached)?;
		Ok(handle)
	}

	/// Adds a child track to this track.
	pub fn add_sub_track(
		&mut self,
		builder: TrackBuilder,
	) -> Result<TrackHandle, ResourceLimitReached> {
		let (mut track, handle) = builder.build(self.renderer_shared.clone());
		track.init_effects(self.renderer_shared.sample_rate.load(Ordering::SeqCst));
		self.sub_track_controller.insert(track)?;
		Ok(handle)
	}

	/// Adds a spatial child track to this track.
	pub fn add_spatial_sub_track(
		&mut self,
		listener: impl Into<ListenerId>,
		position: impl Into<Value<mint::Vector3<f32>>>,
		builder: SpatialTrackBuilder,
	) -> Result<SpatialTrackHandle, ResourceLimitReached> {
		let (mut track, handle) = builder.build(
			self.renderer_shared.clone(),
			listener.into(),
			position.into().to_(),
		);
		track.init_effects(self.renderer_shared.sample_rate.load(Ordering::SeqCst));
		self.sub_track_controller.insert(track)?;
		Ok(handle)
	}

	/// Sets the (post-effects) volume of the mixer track.
	pub fn set_volume(&mut self, volume: impl Into<Value<Volume>>, tween: Tween) {
		self.set_volume_command_writer.write(ValueChangeCommand {
			target: volume.into(),
			tween,
		})
	}

	/// Sets the position that audio is produced from.
	pub fn set_position(&mut self, position: impl Into<Value<mint::Vector3<f32>>>, tween: Tween) {
		let position: Value<mint::Vector3<f32>> = position.into();
		self.set_position_command_writer.write(ValueChangeCommand {
			target: position.to_(),
			tween,
		})
	}

	/// Sets the volume of this track's route to a send track.
	///
	/// This can only be used to change the volume of existing routes,
	/// not to add new routes.
	pub fn set_send(
		&mut self,
		to: impl Into<SendTrackId>,
		volume: impl Into<Value<Volume>>,
		tween: Tween,
	) -> Result<(), NonexistentRoute> {
		let to = to.into();
		self.send_volume_command_writers
			.get_mut(&to)
			.ok_or(NonexistentRoute)?
			.write(ValueChangeCommand {
				target: volume.into(),
				tween,
			});
		Ok(())
	}

	/// Returns the maximum number of sounds that can play simultaneously on this track.
	#[must_use]
	pub fn sound_capacity(&self) -> u16 {
		self.sound_controller.capacity()
	}

	/// Returns the number of sounds currently playing on this track.
	#[must_use]
	pub fn num_sounds(&self) -> u16 {
		self.sound_controller.len()
	}

	/// Returns the maximum number of child tracks this track can have.
	#[must_use]
	pub fn sub_track_capacity(&self) -> u16 {
		self.sub_track_controller.capacity()
	}

	/// Returns the number of child tracks this track has.
	#[must_use]
	pub fn num_sub_tracks(&self) -> u16 {
		self.sub_track_controller.len()
	}
}

impl Drop for SpatialTrackHandle {
	fn drop(&mut self) {
		if let Some(shared) = &self.shared {
			shared.mark_for_removal();
		}
	}
}
