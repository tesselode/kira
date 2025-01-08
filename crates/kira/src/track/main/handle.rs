use crate::{
	backend::resources::ResourceController,
	command::{CommandWriter, ValueChangeCommand},
	sound::{Sound, SoundData},
	Decibels, PlaySoundError, Tween, Value,
};

/// Controls the main mixer track.
#[derive(Debug)]
pub struct MainTrackHandle {
	pub(crate) set_volume_command_writer: CommandWriter<ValueChangeCommand<Decibels>>,
	pub(crate) sound_controller: ResourceController<Box<dyn Sound>>,
}

impl MainTrackHandle {
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

	/// Sets the (post-effects) volume of the mixer track.
	pub fn set_volume(&mut self, volume: impl Into<Value<Decibels>>, tween: Tween) {
		self.set_volume_command_writer.write(ValueChangeCommand {
			target: volume.into(),
			tween,
		})
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
}
