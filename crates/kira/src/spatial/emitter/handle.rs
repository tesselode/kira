use std::sync::Arc;

use crate::{
	command::ValueChangeCommand,
	manager::backend::resources::ResourceController,
	sound::{Sound, SoundData},
	tween::{Tween, Value},
	PlaySoundError,
};

use super::{CommandWriters, EmitterShared};

/// Controls a emitter.
///
/// When a [`EmitterHandle`] is dropped, the corresponding
/// emitter will be removed.
#[derive(Debug)]
pub struct EmitterHandle {
	pub(crate) shared: Arc<EmitterShared>,
	pub(crate) command_writers: CommandWriters,
	pub(crate) sound_controller: ResourceController<Box<dyn Sound>>,
}

impl EmitterHandle {
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

	/// Sets the position that audio is produced from.
	pub fn set_position(&mut self, position: impl Into<Value<mint::Vector3<f32>>>, tween: Tween) {
		let position: Value<mint::Vector3<f32>> = position.into();
		self.command_writers.set_position.write(ValueChangeCommand {
			target: position.to_(),
			tween,
		})
	}
}

impl Drop for EmitterHandle {
	fn drop(&mut self) {
		self.shared.mark_for_removal();
	}
}
