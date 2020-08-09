use crate::{
	command::Command,
	instance::{InstanceId, InstanceSettings},
	metronome::{Metronome, MetronomeId},
	sound::SoundId,
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

static NEXT_SEQUENCE_INDEX: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct SequenceId {
	index: usize,
}

impl SequenceId {
	pub fn new() -> Self {
		let index = NEXT_SEQUENCE_INDEX.fetch_add(1, Ordering::Relaxed);
		Self { index }
	}
}

#[derive(Debug, Clone)]
pub struct SequenceInstanceSettings {
	pub volume: f32,
	pub pitch: f32,
	pub position: Time,
}

impl SequenceInstanceSettings {
	fn into_instance_settings(&self, tempo: f32) -> InstanceSettings {
		InstanceSettings {
			volume: self.volume,
			pitch: self.pitch,
			position: self.position.in_seconds(tempo),
			fade_in_duration: None,
		}
	}
}

impl Default for SequenceInstanceSettings {
	fn default() -> Self {
		Self {
			volume: 1.0,
			pitch: 1.0,
			position: Time::Seconds(0.0),
		}
	}
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) enum SequenceState {
	Idle,
	Playing(usize),
	Finished,
}

#[derive(Debug, Clone)]
enum SequenceCommand {
	Wait(Time),
	WaitForInterval(f32),
	GoTo(usize),
	PlaySound(SoundId, SequenceInstanceHandle, SequenceInstanceSettings),
	PauseInstance(SequenceInstanceHandle, Option<Time>),
	ResumeInstance(SequenceInstanceHandle, Option<Time>),
	StopInstance(SequenceInstanceHandle, Option<Time>),
}

#[derive(Debug, Clone)]
pub struct Sequence {
	pub metronome_id: MetronomeId,
	commands: Vec<SequenceCommand>,
	state: SequenceState,
	wait_timer: Option<f32>,
	instances: HashMap<SequenceInstanceHandle, InstanceId>,
}

impl Sequence {
	pub fn new(metronome_id: MetronomeId) -> Self {
		Self {
			metronome_id,
			commands: vec![],
			state: SequenceState::Idle,
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
		settings: SequenceInstanceSettings,
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

	pub fn pause_instance(&mut self, handle: SequenceInstanceHandle, fade_duration: Option<Time>) {
		self.commands
			.push(SequenceCommand::PauseInstance(handle, fade_duration));
	}

	pub fn resume_instance(&mut self, handle: SequenceInstanceHandle, fade_duration: Option<Time>) {
		self.commands
			.push(SequenceCommand::ResumeInstance(handle, fade_duration));
	}

	pub fn stop_instance(&mut self, handle: SequenceInstanceHandle, fade_duration: Option<Time>) {
		self.commands
			.push(SequenceCommand::StopInstance(handle, fade_duration));
	}

	pub fn go_to(&mut self, index: usize) {
		self.commands.push(SequenceCommand::GoTo(index));
	}

	fn go_to_command(
		&mut self,
		index: usize,
		metronome: &Metronome,
		command_queue: &mut Vec<Command>,
	) {
		if index >= self.commands.len() {
			self.state = SequenceState::Finished;
			return;
		}
		self.state = SequenceState::Playing(index);
		if let Some(command) = self.commands.get(index) {
			let command = command.clone();
			if let SequenceCommand::Wait(_) = command {
				self.wait_timer = Some(1.0);
			} else {
				self.wait_timer = None;
			}
			match command {
				SequenceCommand::GoTo(index) => {
					self.go_to_command(index, metronome, command_queue);
				}
				SequenceCommand::PlaySound(sound_id, sequence_instance_handle, settings) => {
					let instance_id = InstanceId::new();
					self.instances.insert(sequence_instance_handle, instance_id);
					command_queue.push(Command::PlaySound(
						sound_id,
						instance_id,
						settings.into_instance_settings(metronome.tempo),
					));
					self.go_to_command(index + 1, metronome, command_queue);
				}
				SequenceCommand::PauseInstance(handle, fade_duration) => {
					if let Some(instance_id) = self.instances.get(&handle) {
						let fade_duration = match fade_duration {
							Some(time) => Some(time.in_seconds(metronome.tempo)),
							None => None,
						};
						command_queue.push(Command::PauseInstance(*instance_id, fade_duration))
					}
					self.go_to_command(index + 1, metronome, command_queue);
				}
				SequenceCommand::ResumeInstance(handle, fade_duration) => {
					if let Some(instance_id) = self.instances.get(&handle) {
						let fade_duration = match fade_duration {
							Some(time) => Some(time.in_seconds(metronome.tempo)),
							None => None,
						};
						command_queue.push(Command::ResumeInstance(*instance_id, fade_duration))
					}
					self.go_to_command(index + 1, metronome, command_queue);
				}
				SequenceCommand::StopInstance(handle, fade_duration) => {
					if let Some(instance_id) = self.instances.get(&handle) {
						let fade_duration = match fade_duration {
							Some(time) => Some(time.in_seconds(metronome.tempo)),
							None => None,
						};
						command_queue.push(Command::StopInstance(*instance_id, fade_duration))
					}
					self.go_to_command(index + 1, metronome, command_queue);
				}
				_ => {}
			}
		}
	}

	pub(crate) fn start(&mut self, metronome: &Metronome, command_queue: &mut Vec<Command>) {
		self.go_to_command(0, metronome, command_queue);
	}

	pub(crate) fn update(
		&mut self,
		dt: f32,
		metronome: &Metronome,
		command_queue: &mut Vec<Command>,
	) {
		if let SequenceState::Playing(index) = self.state {
			if let Some(command) = self.commands.get(index) {
				match command {
					SequenceCommand::Wait(time) => {
						let time = time.in_seconds(metronome.effective_tempo());
						if let Some(wait_timer) = self.wait_timer.as_mut() {
							*wait_timer -= dt / time;
							if *wait_timer <= 0.0 {
								self.go_to_command(index + 1, metronome, command_queue);
							}
						}
					}
					SequenceCommand::WaitForInterval(interval) => {
						if metronome.interval_passed(*interval) {
							self.go_to_command(index + 1, metronome, command_queue);
						}
					}
					_ => {}
				}
			}
		}
	}

	pub(crate) fn finished(&self) -> bool {
		self.state == SequenceState::Finished
	}
}
