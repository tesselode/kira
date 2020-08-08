use crate::{
	command::Command,
	id::{InstanceId, SoundId},
	manager::InstanceSettings,
	metronome::Metronome,
	time::Time,
};

#[derive(Copy, Clone)]
enum SequenceCommand {
	Wait(Time),
	WaitForInterval(f32),
	GoTo(usize),
	Do(Command),
}

pub struct Sequence {
	commands: Vec<SequenceCommand>,
	playing: bool,
	current_command_index: Option<usize>,
	wait_timer: Option<f32>,
}

impl Sequence {
	pub fn new() -> Self {
		Self {
			commands: vec![],
			playing: true,
			current_command_index: None,
			wait_timer: None,
		}
	}

	pub fn wait(&mut self, time: Time) {
		self.commands.push(SequenceCommand::Wait(time));
	}

	pub fn on_interval(&mut self, interval: f32) {
		self.commands
			.push(SequenceCommand::WaitForInterval(interval));
	}

	pub fn play_sound(&mut self, sound_id: SoundId, settings: InstanceSettings) -> InstanceId {
		let instance_id = InstanceId::new();
		self.commands.push(SequenceCommand::Do(Command::PlaySound(
			sound_id,
			instance_id,
			settings,
		)));
		instance_id
	}

	fn go_to_command(&mut self, index: usize, command_queue: &mut Vec<Command>) {
		self.current_command_index = Some(index);
		if let Some(command) = self.commands.get(index) {
			let command = *command;
			if let SequenceCommand::Wait(_) = command {
				self.wait_timer = Some(1.0);
			} else {
				self.wait_timer = None;
			}
			match command {
				SequenceCommand::GoTo(index) => {
					self.go_to_command(index, command_queue);
				}
				SequenceCommand::Do(command) => {
					command_queue.push(command);
					self.go_to_command(index + 1, command_queue);
				}
				_ => {}
			}
		}
	}

	fn go_to_next_command(&mut self, command_queue: &mut Vec<Command>) {
		if let Some(index) = self.current_command_index {
			self.go_to_command(index + 1, command_queue);
		}
	}

	pub(crate) fn update(
		&mut self,
		dt: f32,
		metronome: &Metronome,
		command_queue: &mut Vec<Command>,
	) {
		if !self.playing {
			return;
		}
		if let Some(index) = self.current_command_index {
			while let Some(command) = self.commands.get(index) {
				match command {
					SequenceCommand::Wait(time) => {
						let time = time.in_seconds(metronome.tempo);
						if let Some(wait_timer) = self.wait_timer.as_mut() {
							*wait_timer -= dt / time;
							if *wait_timer <= 0.0 {
								self.go_to_next_command(command_queue);
							}
						}
					}
					SequenceCommand::WaitForInterval(interval) => {
						if metronome.interval_passed(*interval) {
							self.go_to_next_command(command_queue);
						}
					}
					_ => {}
				}
			}
		}
	}
}
