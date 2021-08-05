use std::{error::Error, fmt::Display, sync::Arc};

use crate::{
	error::CommandError,
	manager::command::{producer::CommandProducer, Command, InstanceCommand},
	parameter::Tween,
	value::Value,
};

use super::{InstanceId, InstanceShared, InstanceState};

/// An error that can occur when modifying an instance.
#[derive(Debug)]
pub enum InstanceHandleError {
	/// Cannot modify an instance that has finished playing.
	InstanceStopped,
	/// An error occured when sending a command to the renderer.
	CommandError(CommandError),
}

impl Display for InstanceHandleError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			InstanceHandleError::InstanceStopped => {
				f.write_str("Cannot modify an instance that has finished playing")
			}
			InstanceHandleError::CommandError(error) => error.fmt(f),
		}
	}
}

impl Error for InstanceHandleError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			InstanceHandleError::CommandError(error) => Some(error),
			_ => None,
		}
	}
}

impl From<CommandError> for InstanceHandleError {
	fn from(v: CommandError) -> Self {
		Self::CommandError(v)
	}
}

/// Controls an instance of a sound.
///
/// Unlike other handles, dropping an [`InstanceHandle`] does **not**
/// cause the corresponding instance to be removed.
pub struct InstanceHandle {
	pub(crate) id: InstanceId,
	pub(crate) shared: Arc<InstanceShared>,
	pub(crate) command_producer: CommandProducer,
}

impl InstanceHandle {
	/// Returns the unique identifier for the instance.
	pub fn id(&self) -> InstanceId {
		self.id
	}

	/// Returns the current playback state of the instance.
	pub fn state(&self) -> InstanceState {
		self.shared.state()
	}

	/// Returns the current playback position of the instance.
	pub fn position(&self) -> f64 {
		self.shared.position()
	}

	/// Sets the volume of the instance.
	pub fn set_volume(&mut self, volume: impl Into<Value>) -> Result<(), InstanceHandleError> {
		if self.state() == InstanceState::Stopped {
			return Err(InstanceHandleError::InstanceStopped);
		}
		self.command_producer
			.push(Command::Instance(InstanceCommand::SetVolume(
				self.id,
				volume.into(),
			)))?;
		Ok(())
	}

	/// Sets the playback rate of the instance, as a factor of the
	/// normal playback rate.
	///
	/// Changing the playback rate will change both the speed
	/// and the pitch of the sound.
	pub fn set_playback_rate(
		&mut self,
		playback_rate: impl Into<Value>,
	) -> Result<(), InstanceHandleError> {
		if self.state() == InstanceState::Stopped {
			return Err(InstanceHandleError::InstanceStopped);
		}
		self.command_producer
			.push(Command::Instance(InstanceCommand::SetPlaybackRate(
				self.id,
				playback_rate.into(),
			)))?;
		Ok(())
	}

	/// Sets the panning of the instance, where 0 is hard left
	/// and 1 is hard right.
	pub fn set_panning(&mut self, panning: impl Into<Value>) -> Result<(), InstanceHandleError> {
		if self.state() == InstanceState::Stopped {
			return Err(InstanceHandleError::InstanceStopped);
		}
		self.command_producer
			.push(Command::Instance(InstanceCommand::SetPanning(
				self.id,
				panning.into(),
			)))?;
		Ok(())
	}

	/// Fades out the instance with the specified tween and then
	/// pauses playback.
	pub fn pause(&mut self, fade_out_tween: Tween) -> Result<(), InstanceHandleError> {
		if self.state() == InstanceState::Stopped {
			return Err(InstanceHandleError::InstanceStopped);
		}
		self.command_producer
			.push(Command::Instance(InstanceCommand::Pause {
				id: self.id,
				tween: fade_out_tween,
			}))?;
		Ok(())
	}

	/// Resume playback of the instance, fading in the audio
	/// with the specified tween.
	pub fn resume(&mut self, fade_in_tween: Tween) -> Result<(), InstanceHandleError> {
		if self.state() == InstanceState::Stopped {
			return Err(InstanceHandleError::InstanceStopped);
		}
		self.command_producer
			.push(Command::Instance(InstanceCommand::Resume {
				id: self.id,
				tween: fade_in_tween,
			}))?;
		Ok(())
	}

	/// Fades out the instance with the specified tween and then
	/// stops playback.
	///
	/// Once the instance has stopped, it can no longer be
	/// interacted with.
	pub fn stop(&mut self, fade_out_tween: Tween) -> Result<(), InstanceHandleError> {
		if self.state() == InstanceState::Stopped {
			return Err(InstanceHandleError::InstanceStopped);
		}
		self.command_producer
			.push(Command::Instance(InstanceCommand::Stop {
				id: self.id,
				tween: fade_out_tween,
			}))?;
		Ok(())
	}

	pub fn seek_to(&mut self, position: f64) -> Result<(), InstanceHandleError> {
		if self.state() == InstanceState::Stopped {
			return Err(InstanceHandleError::InstanceStopped);
		}
		self.command_producer
			.push(Command::Instance(InstanceCommand::SeekTo(
				self.id, position,
			)))?;
		Ok(())
	}

	pub fn seek_by(&mut self, amount: f64) -> Result<(), InstanceHandleError> {
		if self.state() == InstanceState::Stopped {
			return Err(InstanceHandleError::InstanceStopped);
		}
		self.command_producer
			.push(Command::Instance(InstanceCommand::SeekBy(self.id, amount)))?;
		Ok(())
	}
}
