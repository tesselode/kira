use crate::{
	command::{Command, InstanceCommand, MetronomeCommand},
	duration::Duration,
	instance::{InstanceId, InstanceSettings},
	metronome::Metronome,
	sound::SoundId,
	tween::Tween,
};
use std::{
	collections::HashMap,
	sync::atomic::{AtomicUsize, Ordering},
};

static NEXT_SEQUENCE_INSTANCE_HANDLE_INDEX: AtomicUsize = AtomicUsize::new(0);

/// A handle to a "play sound" task in a sequence.
///
/// This can be used to pause or resume an instance in a
/// later task in the sequence.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct SequenceInstanceHandle {
	index: usize,
}

impl SequenceInstanceHandle {
	pub fn new() -> Self {
		let index = NEXT_SEQUENCE_INSTANCE_HANDLE_INDEX.fetch_add(1, Ordering::Relaxed);
		Self { index }
	}
}

static NEXT_SEQUENCE_INDEX: AtomicUsize = AtomicUsize::new(0);

/// A unique identifier for a `Sequence`.
///
/// You cannot create this manually - a `SequenceId` is returned
/// when you start a sequence with an `AudioManager`.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct SequenceId {
	index: usize,
}

impl SequenceId {
	pub(crate) fn new() -> Self {
		let index = NEXT_SEQUENCE_INDEX.fetch_add(1, Ordering::Relaxed);
		Self { index }
	}
}

#[derive(Debug, Copy, Clone)]
enum SequenceCommand {
	Instance(InstanceCommand<SequenceInstanceHandle>),
	Metronome(MetronomeCommand),
}

pub(crate) enum SequenceOutputCommand {
	Instance(InstanceCommand<InstanceId>),
	Metronome(MetronomeCommand),
}

impl Into<Command> for SequenceOutputCommand {
	fn into(self) -> Command {
		match self {
			SequenceOutputCommand::Instance(command) => Command::Instance(command),
			SequenceOutputCommand::Metronome(command) => Command::Metronome(command),
		}
	}
}

#[derive(Debug, Copy, Clone)]
enum SequenceTask {
	Wait(Duration),
	WaitForInterval(f32),
	GoToTask(usize),
	RunCommand(SequenceCommand),
}

#[derive(Debug, Clone)]
enum SequenceState {
	Idle,
	Playing(usize),
	Finished,
}

#[derive(Debug, Clone)]
pub struct Sequence {
	tasks: Vec<SequenceTask>,
	state: SequenceState,
	wait_timer: Option<f32>,
	instances: HashMap<SequenceInstanceHandle, InstanceId>,
}

impl Sequence {
	pub fn new() -> Self {
		Self {
			tasks: vec![],
			state: SequenceState::Idle,
			wait_timer: None,
			instances: HashMap::new(),
		}
	}

	pub fn wait(&mut self, duration: Duration) {
		self.tasks.push(SequenceTask::Wait(duration));
	}

	pub fn wait_for_interval(&mut self, interval: f32) {
		self.tasks.push(SequenceTask::WaitForInterval(interval));
	}

	pub fn go_to(&mut self, index: usize) {
		self.tasks.push(SequenceTask::GoToTask(index));
	}

	pub fn play_sound(
		&mut self,
		sound_id: SoundId,
		settings: InstanceSettings,
	) -> SequenceInstanceHandle {
		let handle = SequenceInstanceHandle::new();
		self.tasks
			.push(SequenceTask::RunCommand(SequenceCommand::Instance(
				InstanceCommand::PlaySound(sound_id, handle, settings),
			)));
		handle
	}

	pub fn set_instance_volume(
		&mut self,
		handle: SequenceInstanceHandle,
		volume: f32,
		tween: Option<Tween>,
	) {
		self.tasks
			.push(SequenceTask::RunCommand(SequenceCommand::Instance(
				InstanceCommand::SetInstanceVolume(handle, volume, tween),
			)));
	}

	pub fn set_instance_pitch(
		&mut self,
		handle: SequenceInstanceHandle,
		pitch: f32,
		tween: Option<Tween>,
	) {
		self.tasks
			.push(SequenceTask::RunCommand(SequenceCommand::Instance(
				InstanceCommand::SetInstancePitch(handle, pitch, tween),
			)));
	}

	pub fn pause_instance(&mut self, handle: SequenceInstanceHandle, fade_tween: Option<Tween>) {
		self.tasks
			.push(SequenceTask::RunCommand(SequenceCommand::Instance(
				InstanceCommand::PauseInstance(handle, fade_tween),
			)));
	}

	pub fn resume_instance(&mut self, handle: SequenceInstanceHandle, fade_tween: Option<Tween>) {
		self.tasks
			.push(SequenceTask::RunCommand(SequenceCommand::Instance(
				InstanceCommand::ResumeInstance(handle, fade_tween),
			)));
	}

	pub fn stop_instance(&mut self, handle: SequenceInstanceHandle, fade_tween: Option<Tween>) {
		self.tasks
			.push(SequenceTask::RunCommand(SequenceCommand::Instance(
				InstanceCommand::StopInstance(handle, fade_tween),
			)));
	}

	pub fn pause_instances_of_sound(&mut self, id: SoundId, fade_tween: Option<Tween>) {
		self.tasks
			.push(SequenceTask::RunCommand(SequenceCommand::Instance(
				InstanceCommand::PauseInstancesOfSound(id, fade_tween),
			)));
	}

	pub fn resume_instances_of_sound(&mut self, id: SoundId, fade_tween: Option<Tween>) {
		self.tasks
			.push(SequenceTask::RunCommand(SequenceCommand::Instance(
				InstanceCommand::ResumeInstancesOfSound(id, fade_tween),
			)));
	}

	pub fn stop_instances_of_sound(&mut self, id: SoundId, fade_tween: Option<Tween>) {
		self.tasks
			.push(SequenceTask::RunCommand(SequenceCommand::Instance(
				InstanceCommand::StopInstancesOfSound(id, fade_tween),
			)));
	}

	pub fn start_metronome(&mut self) {
		self.tasks
			.push(SequenceTask::RunCommand(SequenceCommand::Metronome(
				MetronomeCommand::StartMetronome,
			)));
	}

	pub fn pause_metronome(&mut self) {
		self.tasks
			.push(SequenceTask::RunCommand(SequenceCommand::Metronome(
				MetronomeCommand::PauseMetronome,
			)));
	}

	pub fn stop_metronome(&mut self) {
		self.tasks
			.push(SequenceTask::RunCommand(SequenceCommand::Metronome(
				MetronomeCommand::StopMetronome,
			)));
	}

	fn start_task(&mut self, index: usize) {
		if let Some(task) = self.tasks.get(index) {
			self.state = SequenceState::Playing(index);
			if let SequenceTask::Wait(_) = task {
				self.wait_timer = Some(1.0);
			} else {
				self.wait_timer = None;
			}
		} else {
			self.state = SequenceState::Finished;
		}
	}

	pub(crate) fn start(&mut self) {
		self.start_task(0);
	}

	fn transform_command(&mut self, command: SequenceCommand) -> SequenceOutputCommand {
		match command {
			SequenceCommand::Instance(command) => match command {
				InstanceCommand::PlaySound(sound_id, handle, settings) => {
					let instance_id = InstanceId::new();
					self.instances.insert(handle, instance_id);
					SequenceOutputCommand::Instance(InstanceCommand::PlaySound(
						sound_id,
						instance_id,
						settings,
					))
				}
				InstanceCommand::SetInstanceVolume(handle, volume, tween) => {
					let instance_id = self.instances.get(&handle).unwrap();
					SequenceOutputCommand::Instance(InstanceCommand::SetInstanceVolume(
						*instance_id,
						volume,
						tween,
					))
				}
				InstanceCommand::SetInstancePitch(handle, pitch, tween) => {
					let instance_id = self.instances.get(&handle).unwrap();
					SequenceOutputCommand::Instance(InstanceCommand::SetInstancePitch(
						*instance_id,
						pitch,
						tween,
					))
				}
				InstanceCommand::PauseInstance(handle, fade_tween) => {
					let instance_id = self.instances.get(&handle).unwrap();
					SequenceOutputCommand::Instance(InstanceCommand::PauseInstance(
						*instance_id,
						fade_tween,
					))
				}
				InstanceCommand::ResumeInstance(handle, fade_tween) => {
					let instance_id = self.instances.get(&handle).unwrap();
					SequenceOutputCommand::Instance(InstanceCommand::ResumeInstance(
						*instance_id,
						fade_tween,
					))
				}
				InstanceCommand::StopInstance(handle, fade_tween) => {
					let instance_id = self.instances.get(&handle).unwrap();
					SequenceOutputCommand::Instance(InstanceCommand::StopInstance(
						*instance_id,
						fade_tween,
					))
				}
				InstanceCommand::PauseInstancesOfSound(id, fade_tween) => {
					SequenceOutputCommand::Instance(InstanceCommand::PauseInstancesOfSound(
						id, fade_tween,
					))
				}
				InstanceCommand::ResumeInstancesOfSound(id, fade_tween) => {
					SequenceOutputCommand::Instance(InstanceCommand::ResumeInstancesOfSound(
						id, fade_tween,
					))
				}
				InstanceCommand::StopInstancesOfSound(id, fade_tween) => {
					SequenceOutputCommand::Instance(InstanceCommand::StopInstancesOfSound(
						id, fade_tween,
					))
				}
			},
			SequenceCommand::Metronome(command) => SequenceOutputCommand::Metronome(command),
		}
	}

	pub(crate) fn update(
		&mut self,
		dt: f32,
		metronome: &Metronome,
		output_command_queue: &mut Vec<SequenceOutputCommand>,
	) {
		while let SequenceState::Playing(index) = self.state {
			if let Some(task) = self.tasks.get(index) {
				let task = *task;
				match task {
					SequenceTask::Wait(duration) => {
						if let Some(time) = self.wait_timer.as_mut() {
							let duration = duration.in_seconds(metronome.effective_tempo());
							*time -= dt / duration;
							if *time <= 0.0 {
								self.start_task(index + 1);
							}
							break;
						}
					}
					SequenceTask::WaitForInterval(interval) => {
						if metronome.interval_passed(interval) {
							self.start_task(index + 1);
						}
						break;
					}
					SequenceTask::GoToTask(index) => {
						self.start_task(index);
					}
					SequenceTask::RunCommand(command) => {
						output_command_queue.push(self.transform_command(command));
						self.start_task(index + 1);
					}
				}
			}
		}
	}

	pub(crate) fn finished(&self) -> bool {
		if let SequenceState::Finished = self.state {
			true
		} else {
			false
		}
	}
}
