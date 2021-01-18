//! An interface for controlling metronomes.

use flume::{Receiver, Sender, TryIter};
use thiserror::Error;

use crate::{
	command::{Command, MetronomeCommand},
	Tempo, Value,
};

use super::MetronomeId;

/// Something that can go wrong when using a [`MetronomeHandle`]
/// to control a metronome.
#[derive(Debug, Error)]
pub enum MetronomeHandleError {
	/// The audio thread has finished and can no longer receive commands.
	#[error("The backend cannot receive commands because it no longer exists")]
	BackendDisconnected,
}

/// Allows you to control a metronome.
pub struct MetronomeHandle {
	id: MetronomeId,
	command_sender: Sender<Command>,
	event_receiver: Receiver<f64>,
}

impl MetronomeHandle {
	pub(crate) fn new(
		id: MetronomeId,
		command_sender: Sender<Command>,
		event_receiver: Receiver<f64>,
	) -> Self {
		Self {
			id,
			command_sender,
			event_receiver,
		}
	}

	/// Gets the ID of the metronome.
	pub fn id(&self) -> MetronomeId {
		self.id
	}

	/// Sets the tempo of the metronome (in beats per minute).
	pub fn set_tempo(
		&mut self,
		tempo: impl Into<Value<Tempo>>,
	) -> Result<(), MetronomeHandleError> {
		self.command_sender
			.send(MetronomeCommand::SetMetronomeTempo(self.id(), tempo.into()).into())
			.map_err(|_| MetronomeHandleError::BackendDisconnected)
	}

	/// Starts the metronome.
	pub fn start(&mut self) -> Result<(), MetronomeHandleError> {
		self.command_sender
			.send(MetronomeCommand::StartMetronome(self.id()).into())
			.map_err(|_| MetronomeHandleError::BackendDisconnected)
	}

	/// Pauses the metronome.
	pub fn pause(&mut self) -> Result<(), MetronomeHandleError> {
		self.command_sender
			.send(MetronomeCommand::PauseMetronome(self.id()).into())
			.map_err(|_| MetronomeHandleError::BackendDisconnected)
	}

	/// Stops the metronome and resets its time to zero.
	pub fn stop(&mut self) -> Result<(), MetronomeHandleError> {
		self.command_sender
			.send(MetronomeCommand::StopMetronome(self.id()).into())
			.map_err(|_| MetronomeHandleError::BackendDisconnected)
	}

	/// Returns an iterator over new interval events
	/// that this metronome emitted.
	pub fn event_iter(&mut self) -> TryIter<f64> {
		self.event_receiver.try_iter()
	}
}
