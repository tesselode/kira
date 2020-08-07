use crate::{
	manager::{InstanceSettings, LooperSettings},
	project::SoundId,
	time::Time,
};

#[derive(Debug)]
pub(crate) enum SequenceCommand {
	OnInterval(f32),
	Wait(Time),
	PlaySound(SoundId, InstanceSettings),
	LoopSound(SoundId, LooperSettings),
}

pub struct Sequence {
	pub(crate) commands: Vec<SequenceCommand>,
	pub(crate) wait_timer: Option<f32>,
}

impl Sequence {
	pub fn new() -> Self {
		Self {
			commands: vec![],
			wait_timer: None,
		}
	}

	pub fn on_interval(mut self, interval: f32) -> Self {
		self.commands.push(SequenceCommand::OnInterval(interval));
		self
	}

	pub fn wait(mut self, time: Time) -> Self {
		self.commands.push(SequenceCommand::Wait(time));
		if self.commands.len() == 1 {
			self.wait_timer = Some(1.0);
		}
		self
	}

	pub fn play_sound(mut self, sound_id: SoundId, settings: InstanceSettings) -> Self {
		self.commands
			.push(SequenceCommand::PlaySound(sound_id, settings));
		self
	}

	pub fn loop_sound(mut self, sound_id: SoundId, settings: LooperSettings) -> Self {
		self.commands
			.push(SequenceCommand::LoopSound(sound_id, settings));
		self
	}

	pub(crate) fn goto_next(&mut self) {
		self.commands.remove(0);
		if let Some(command) = self.commands.first() {
			match command {
				SequenceCommand::Wait(_) => {
					self.wait_timer = Some(1.0);
				}
				_ => {
					self.wait_timer = None;
				}
			}
		}
	}
}
