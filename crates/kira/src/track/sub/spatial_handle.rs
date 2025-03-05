use std::{
	collections::HashMap,
	sync::{atomic::Ordering, Arc},
};

use crate::{
	backend::{resources::ResourceController, RendererShared},
	command::{CommandWriter, ValueChangeCommand},
	listener::ListenerId,
	sound::{Sound, SoundData},
	track::TrackPlaybackState,
	Decibels, PlaySoundError, ResourceLimitReached, StartTime, Tween, Value,
};

use super::{
	CommandWriters, NonexistentRoute, SendTrackId, SpatialTrackBuilder, Track, TrackBuilder,
	TrackHandle, TrackShared,
};

/// Controls a mixer track.
///
/// When a [`SpatialTrackHandle`] is dropped, the corresponding mixer
/// track will be removed.
#[derive(Debug)]
pub struct SpatialTrackHandle {
	pub(crate) renderer_shared: Arc<RendererShared>,
	pub(crate) shared: Arc<TrackShared>,
	pub(crate) command_writers: CommandWriters,
	pub(crate) sound_controller: ResourceController<Box<dyn Sound>>,
	pub(crate) sub_track_controller: ResourceController<Track>,
	pub(crate) send_volume_command_writers:
		HashMap<SendTrackId, CommandWriter<ValueChangeCommand<Decibels>>>,
	pub(crate) internal_buffer_size: usize,
}

impl SpatialTrackHandle {
	/// Returns the current playback state of the track.
	#[must_use]
	pub fn state(&self) -> TrackPlaybackState {
		self.shared.state()
	}

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
		let (mut track, handle) =
			builder.build(self.renderer_shared.clone(), self.internal_buffer_size);
		track.init_effects(self.renderer_shared.sample_rate.load(Ordering::SeqCst));
		self.sub_track_controller.insert(track)?;
		Ok(handle)
	}

	/// Adds a spatial child track to this track.
	pub fn add_spatial_sub_track(
		&mut self,
		listener: impl Into<ListenerId>,
		position: impl Into<Value<mint::Vector3<f32>>>,
		game_loop_delta_time: impl Into<Value<f64>>,
		builder: SpatialTrackBuilder,
	) -> Result<SpatialTrackHandle, ResourceLimitReached> {
		let (mut track, handle) = builder.build(
			self.renderer_shared.clone(),
			self.internal_buffer_size,
			listener.into(),
			position.into().to_(),
			game_loop_delta_time.into().to_(),
		);
		track.init_effects(self.renderer_shared.sample_rate.load(Ordering::SeqCst));
		self.sub_track_controller.insert(track)?;
		Ok(handle)
	}

	/// Sets the (post-effects) volume of the mixer track.
	pub fn set_volume(&mut self, volume: impl Into<Value<Decibels>>, tween: Tween) {
		self.command_writers.set_volume.write(ValueChangeCommand {
			target: volume.into(),
			tween,
		})
	}

	/// Sets the position that audio is produced from.
	pub fn set_position(&mut self, position: impl Into<Value<mint::Vector3<f32>>>, tween: Tween) {
		let position: Value<mint::Vector3<f32>> = position.into();
		self.command_writers.set_position.write(ValueChangeCommand {
			target: position.to_(),
			tween,
		})
	}

	/// Sets the delta time of the game loop. Needed for things like the doppler effect.
	pub fn set_game_loop_delta_time(&mut self, game_loop_delta_time: f64) {
		let game_loop_delta_time: Value<f64> = game_loop_delta_time.into();
		self.command_writers.set_game_loop_delta_time.write(ValueChangeCommand {
			target: game_loop_delta_time.to_(),
			tween: Tween::default(),
		})
	}

	/// Sets how much the track's output should be panned left or right depending on its
	/// direction from the listener.
	///
	/// This value should be between `0.0` and `1.0`. `0.0` disables spatialization
	/// entirely.
	pub fn set_spatialization_strength(
		&mut self,
		spatialization_strength: impl Into<Value<f32>>,
		tween: Tween,
	) {
		self.command_writers
			.set_spatialization_strength
			.write(ValueChangeCommand {
				target: spatialization_strength.into(),
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
		volume: impl Into<Value<Decibels>>,
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

	/// Fades out the track to silence with the given tween and then
	/// pauses playback, pausing all sounds and emitters playing on this
	/// track.
	pub fn pause(&mut self, tween: Tween) {
		self.command_writers.pause.write(tween)
	}

	/// Resumes playback and fades in the sound from silence
	/// with the given tween, resuming all sounds and emitters
	/// playing on this track.
	pub fn resume(&mut self, tween: Tween) {
		self.resume_at(StartTime::Immediate, tween)
	}

	/// Resumes playback at the given start time and fades in
	/// the sound from silence with the given tween.
	pub fn resume_at(&mut self, start_time: StartTime, tween: Tween) {
		self.command_writers.resume.write((start_time, tween))
	}

	/// Returns the maximum number of sounds that can play simultaneously on this track.
	#[must_use]
	pub fn sound_capacity(&self) -> usize {
		self.sound_controller.capacity()
	}

	/// Returns the number of sounds currently playing on this track.
	#[must_use]
	pub fn num_sounds(&self) -> usize {
		self.sound_controller.len()
	}

	/// Returns the maximum number of child tracks this track can have.
	#[must_use]
	pub fn sub_track_capacity(&self) -> usize {
		self.sub_track_controller.capacity()
	}

	/// Returns the number of child tracks this track has.
	#[must_use]
	pub fn num_sub_tracks(&self) -> usize {
		self.sub_track_controller.len()
	}
}

impl Drop for SpatialTrackHandle {
	fn drop(&mut self) {
		self.shared.mark_for_removal();
	}
}
