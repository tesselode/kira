use crate::{
	command::{sender::CommandSender, MetronomeCommand},
	AudioResult, Tempo, Value,
};

use super::MetronomeId;

pub struct MetronomeHandle {
	id: MetronomeId,
	command_sender: CommandSender,
}

impl MetronomeHandle {
	pub(crate) fn new(id: MetronomeId, command_sender: CommandSender) -> Self {
		Self { id, command_sender }
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
}
