use flume::{Receiver, Sender, TryIter};

use crate::{
	command::{Command, MetronomeCommand},
	AudioError, AudioResult, Tempo, Value,
};

use super::MetronomeId;

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

	pub fn id(&self) -> MetronomeId {
		self.id
	}

	pub fn set_tempo(&mut self, tempo: impl Into<Value<Tempo>>) -> AudioResult<()> {
		self.command_sender
			.send(MetronomeCommand::SetMetronomeTempo(self.id(), tempo.into()).into())
			.map_err(|_| AudioError::BackendDisconnected)
	}

	pub fn start(&mut self) -> AudioResult<()> {
		self.command_sender
			.send(MetronomeCommand::StartMetronome(self.id()).into())
			.map_err(|_| AudioError::BackendDisconnected)
	}

	pub fn pause(&mut self) -> AudioResult<()> {
		self.command_sender
			.send(MetronomeCommand::PauseMetronome(self.id()).into())
			.map_err(|_| AudioError::BackendDisconnected)
	}

	pub fn stop(&mut self) -> AudioResult<()> {
		self.command_sender
			.send(MetronomeCommand::StopMetronome(self.id()).into())
			.map_err(|_| AudioError::BackendDisconnected)
	}

	pub fn event_iter(&mut self) -> TryIter<f64> {
		self.event_receiver.try_iter()
	}
}
