use crate::{
	command::{sender::CommandSender, MixerCommand},
	mixer::TrackIndex,
	AudioResult,
};

use super::{EffectId, EffectSettings};

pub struct EffectHandle {
	id: EffectId,
	track_index: TrackIndex,
	enabled: bool,
	command_sender: CommandSender,
}

impl EffectHandle {
	pub(crate) fn new(
		track_index: TrackIndex,
		settings: &EffectSettings,
		command_sender: CommandSender,
	) -> Self {
		Self {
			id: settings.id,
			enabled: settings.enabled,
			track_index,
			command_sender,
		}
	}

	pub fn id(&self) -> EffectId {
		self.id
	}

	pub fn track_index(&self) -> TrackIndex {
		self.track_index
	}

	pub fn enabled(&self) -> bool {
		self.enabled
	}

	pub fn set_enabled(&mut self, enabled: bool) -> AudioResult<()> {
		self.enabled = enabled;
		self.command_sender
			.push(MixerCommand::SetEffectEnabled(self.track_index, self.id, enabled).into())
	}
}
