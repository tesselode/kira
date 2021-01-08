use flume::{Receiver, TryIter};

use crate::{
	command::{sender::CommandSender, MetronomeCommand},
	AudioResult, Tempo, Value,
};

use super::{Metronome, MetronomeId};

pub struct MetronomeHandle {
	id: MetronomeId,
	command_sender: CommandSender,
	event_receiver: Receiver<f64>,
}

impl MetronomeHandle {
	pub(crate) fn new(
		metronome: &Metronome,
		command_sender: CommandSender,
		event_receiver: Receiver<f64>,
	) -> Self {
		Self {
			id: metronome.id(),
			command_sender,
			event_receiver,
		}
	}

	pub fn id(&self) -> MetronomeId {
		self.id
	}

	pub fn set_tempo(&mut self, tempo: impl Into<Value<Tempo>>) -> AudioResult<()> {
		self.command_sender
			.push(MetronomeCommand::SetMetronomeTempo(self.id(), tempo.into()).into())
	}

	pub fn start(&mut self) -> AudioResult<()> {
		self.command_sender
			.push(MetronomeCommand::StartMetronome(self.id()).into())
	}

	pub fn pause(&mut self) -> AudioResult<()> {
		self.command_sender
			.push(MetronomeCommand::PauseMetronome(self.id()).into())
	}

	pub fn stop(&mut self) -> AudioResult<()> {
		self.command_sender
			.push(MetronomeCommand::StopMetronome(self.id()).into())
	}

	pub fn event_iter(&mut self) -> TryIter<f64> {
		self.event_receiver.try_iter()
	}
}
