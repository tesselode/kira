use crate::{
	command::Command,
	id::{InstanceId, MetronomeId, SoundId},
	manager::InstanceSettings,
	metronome::Metronome,
	time::Time,
};
use std::{
	collections::HashMap,
	sync::atomic::{AtomicUsize, Ordering},
};

static NEXT_INSTANCE_HANDLE_INDEX: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct SequenceInstanceHandle {
	index: usize,
}

impl SequenceInstanceHandle {
	pub fn new() -> Self {
		let index = NEXT_INSTANCE_HANDLE_INDEX.fetch_add(1, Ordering::Relaxed);
		Self { index }
	}
}

#[derive(Debug, Clone)]
enum SequenceCommand {
	Wait(Time),
	WaitForInterval(f32),
	GoTo(usize),
	PlaySound(SoundId, SequenceInstanceHandle, InstanceSettings),
}

#[derive(Debug, Clone)]
pub struct Sequence {
	pub metronome_id: MetronomeId,
	commands: Vec<SequenceCommand>,
	playing: bool,
	current_command_index: Option<usize>,
	wait_timer: Option<f32>,
	instances: HashMap<SequenceInstanceHandle, InstanceId>,
}

impl Sequence {
	pub fn new(metronome_id: MetronomeId) -> Self {
		Self {
			metronome_id,
			commands: vec![],
			playing: false,
			current_command_index: None,
			wait_timer: None,
			instances: HashMap::new(),
		}
	}

	pub fn wait(&mut self, time: Time) {
		self.commands.push(SequenceCommand::Wait(time));
	}

	pub fn on_interval(&mut self, interval: f32) {
		self.commands
			.push(SequenceCommand::WaitForInterval(interval));
	}

	pub fn play_sound(
		&mut self,
		sound_id: SoundId,
		settings: InstanceSettings,
	) -> SequenceInstanceHandle {
		self.instances.reserve(1);
		let sequence_instance_handle = SequenceInstanceHandle::new();
		self.commands.push(SequenceCommand::PlaySound(
			sound_id,
			sequence_instance_handle,
			settings,
		));
		sequence_instance_handle
	}

	pub fn go_to(&mut self, index: usize) {
		self.commands.push(SequenceCommand::GoTo(index));
	}

	fn go_to_command(&mut self, index: usize, command_queue: &mut Vec<Command>) {
		self.current_command_index = Some(index);
		if let Some(command) = self.commands.get(index) {
			let command = command.clone();
			if let SequenceCommand::Wait(_) = command {
				self.wait_timer = Some(1.0);
			} else {
				self.wait_timer = None;
			}
			match command {
				SequenceCommand::GoTo(index) => {
					self.go_to_command(index, command_queue);
				}
				SequenceCommand::PlaySound(sound_id, sequence_instance_handle, settings) => {
					let instance_id = InstanceId::new();
					self.instances.insert(sequence_instance_handle, instance_id);
					command_queue.push(Command::PlaySound(sound_id, instance_id, settings));
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

	pub(crate) fn start(&mut self, command_queue: &mut Vec<Command>) {
		self.playing = true;
		self.go_to_command(0, command_queue);
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
			if let Some(command) = self.commands.get(index) {
				match command {
					SequenceCommand::Wait(time) => {
						let time = time.in_seconds(metronome.effective_tempo());
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
