use std::{
	collections::HashMap,
	sync::{atomic::Ordering, Arc},
};

use crate::{
	command::{CommandWriter, ValueChangeCommand},
	manager::backend::{resources::ResourceController, RendererShared},
	sound::{Sound, SoundData},
	track::TrackPlaybackState,
	tween::{Tween, Value},
	Decibels, PlaySoundError, ResourceLimitReached, StartTime,
};

use super::{CommandWriters, NonexistentRoute, SendTrackId, Track, TrackBuilder, TrackShared};

/// Controls a mixer track.
///
/// When a [`TrackHandle`] is dropped, the corresponding mixer
/// track will be removed.
#[derive(Debug)]
pub struct TrackHandle {
	pub(crate) renderer_shared: Arc<RendererShared>,
	pub(crate) shared: Arc<TrackShared>,
	pub(crate) command_writers: CommandWriters,
	pub(crate) sound_controller: ResourceController<Box<dyn Sound>>,
	pub(crate) sub_track_controller: ResourceController<Track>,
	pub(crate) send_volume_command_writers:
		HashMap<SendTrackId, CommandWriter<ValueChangeCommand<Decibels>>>,
}

impl TrackHandle {
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
		let (mut track, handle) = builder.build(self.renderer_shared.clone());
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

impl Drop for TrackHandle {
	fn drop(&mut self) {
		self.shared.mark_for_removal();
	}
}
