use crate::{
	duration::Duration,
	error::ConductorError,
	error::ConductorResult,
	instance::{InstanceId, InstanceSettings},
	metronome::Metronome,
	sound::SoundId,
	tempo::Tempo,
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
pub(crate) enum SequenceOutputCommand<InstanceIdKind, CustomEvent> {
	PlaySound(InstanceIdKind, SoundId, InstanceSettings),
	SetInstanceVolume(InstanceIdKind, f64, Option<Tween>),
	SetInstancePitch(InstanceIdKind, f64, Option<Tween>),
	PauseInstance(InstanceIdKind, Option<Tween>),
	ResumeInstance(InstanceIdKind, Option<Tween>),
	StopInstance(InstanceIdKind, Option<Tween>),
	PauseInstancesOfSound(SoundId, Option<Tween>),
	ResumeInstancesOfSound(SoundId, Option<Tween>),
	StopInstancesOfSound(SoundId, Option<Tween>),
	SetMetronomeTempo(Tempo),
	StartMetronome,
	PauseMetronome,
	StopMetronome,
	EmitCustomEvent(CustomEvent),
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum SequenceTask<InstanceIdKind, CustomEvent> {
	Wait(Duration),
	WaitForInterval(f64),
	RunCommand(SequenceOutputCommand<InstanceIdKind, CustomEvent>),
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum SequenceState {
	Playing,
	Paused,
	Finished,
}

#[derive(Debug, Clone)]
pub struct Sequence<CustomEvent> {
	tasks: Vec<SequenceTask<SequenceInstanceHandle, CustomEvent>>,
	loop_point: Option<usize>,
	state: SequenceState,
	position: usize,
	wait_timer: Option<f64>,
	instances: HashMap<SequenceInstanceHandle, InstanceId>,
	muted: bool,
}

impl<CustomEvent: Copy> Sequence<CustomEvent> {
	pub fn new() -> Self {
		Self {
			tasks: vec![],
			loop_point: None,
			state: SequenceState::Playing,
			position: 0,
			wait_timer: None,
			instances: HashMap::new(),
			muted: false,
		}
	}

	pub fn wait(&mut self, duration: Duration) {
		self.tasks.push(SequenceTask::Wait(duration));
	}

	pub fn wait_for_interval(&mut self, interval: f64) {
		self.tasks.push(SequenceTask::WaitForInterval(interval));
	}

	pub fn start_loop(&mut self) {
		self.loop_point = Some(self.tasks.len())
	}

	pub fn play_sound(
		&mut self,
		sound_id: SoundId,
		settings: InstanceSettings,
	) -> SequenceInstanceHandle {
		let handle = SequenceInstanceHandle::new();
		self.tasks
			.push(SequenceTask::RunCommand(SequenceOutputCommand::PlaySound(
				handle, sound_id, settings,
			)));
		handle
	}

	pub fn set_instance_volume(
		&mut self,
		handle: SequenceInstanceHandle,
		volume: f64,
		tween: Option<Tween>,
	) {
		self.tasks.push(SequenceTask::RunCommand(
			SequenceOutputCommand::SetInstanceVolume(handle, volume, tween),
		));
	}

	pub fn set_instance_pitch(
		&mut self,
		handle: SequenceInstanceHandle,
		pitch: f64,
		tween: Option<Tween>,
	) {
		self.tasks.push(SequenceTask::RunCommand(
			SequenceOutputCommand::SetInstancePitch(handle, pitch, tween),
		));
	}

	pub fn pause_instance(&mut self, handle: SequenceInstanceHandle, fade_tween: Option<Tween>) {
		self.tasks.push(SequenceTask::RunCommand(
			SequenceOutputCommand::PauseInstance(handle, fade_tween),
		));
	}

	pub fn resume_instance(&mut self, handle: SequenceInstanceHandle, fade_tween: Option<Tween>) {
		self.tasks.push(SequenceTask::RunCommand(
			SequenceOutputCommand::ResumeInstance(handle, fade_tween),
		));
	}

	pub fn stop_instance(&mut self, handle: SequenceInstanceHandle, fade_tween: Option<Tween>) {
		self.tasks.push(SequenceTask::RunCommand(
			SequenceOutputCommand::StopInstance(handle, fade_tween),
		));
	}

	pub fn pause_instances_of_sound(&mut self, id: SoundId, fade_tween: Option<Tween>) {
		self.tasks.push(SequenceTask::RunCommand(
			SequenceOutputCommand::PauseInstancesOfSound(id, fade_tween),
		));
	}

	pub fn resume_instances_of_sound(&mut self, id: SoundId, fade_tween: Option<Tween>) {
		self.tasks.push(SequenceTask::RunCommand(
			SequenceOutputCommand::ResumeInstancesOfSound(id, fade_tween),
		));
	}

	pub fn stop_instances_of_sound(&mut self, id: SoundId, fade_tween: Option<Tween>) {
		self.tasks.push(SequenceTask::RunCommand(
			SequenceOutputCommand::StopInstancesOfSound(id, fade_tween),
		));
	}

	pub fn start_metronome(&mut self) {
		self.tasks.push(SequenceTask::RunCommand(
			SequenceOutputCommand::StartMetronome,
		));
	}

	pub fn pause_metronome(&mut self) {
		self.tasks.push(SequenceTask::RunCommand(
			SequenceOutputCommand::PauseMetronome,
		));
	}

	pub fn stop_metronome(&mut self) {
		self.tasks.push(SequenceTask::RunCommand(
			SequenceOutputCommand::StopMetronome,
		));
	}

	pub fn emit_custom_event(&mut self, event: CustomEvent) {
		self.tasks.push(SequenceTask::RunCommand(
			SequenceOutputCommand::EmitCustomEvent(event),
		));
	}

	pub(crate) fn validate(&self) -> ConductorResult<()> {
		if let Some(loop_point) = self.loop_point {
			if loop_point >= self.tasks.len() {
				return Err(ConductorError::InvalidSequenceLoopPoint);
			}
		}
		Ok(())
	}

	fn start_task(&mut self, index: usize) {
		if let Some(task) = self.tasks.get(index) {
			self.position = index;
			if let SequenceTask::Wait(_) = task {
				self.wait_timer = Some(1.0);
			} else {
				self.wait_timer = None;
			}
		} else if let Some(loop_point) = self.loop_point {
			self.start_task(loop_point);
		} else {
			self.state = SequenceState::Finished;
		}
	}

	pub(crate) fn start(&mut self) {
		self.start_task(0);
	}

	pub(crate) fn mute(&mut self) {
		self.muted = true;
	}

	pub(crate) fn unmute(&mut self) {
		self.muted = false;
	}

	pub(crate) fn pause(&mut self) {
		self.state = SequenceState::Paused;
	}

	pub(crate) fn resume(&mut self) {
		self.state = SequenceState::Playing;
	}

	pub(crate) fn stop(&mut self) {
		self.state = SequenceState::Finished;
	}

	fn transform_output_command(
		&mut self,
		task: SequenceOutputCommand<SequenceInstanceHandle, CustomEvent>,
	) -> SequenceOutputCommand<InstanceId, CustomEvent> {
		match task {
			SequenceOutputCommand::PlaySound(handle, sound_id, settings) => {
				let instance_id = InstanceId::new();
				self.instances.insert(handle, instance_id);
				SequenceOutputCommand::PlaySound(instance_id, sound_id, settings)
			}
			SequenceOutputCommand::SetInstanceVolume(handle, volume, tween) => {
				let instance_id = self.instances.get(&handle).unwrap();
				SequenceOutputCommand::SetInstanceVolume(*instance_id, volume, tween)
			}
			SequenceOutputCommand::SetInstancePitch(handle, pitch, tween) => {
				let instance_id = self.instances.get(&handle).unwrap();
				SequenceOutputCommand::SetInstancePitch(*instance_id, pitch, tween)
			}
			SequenceOutputCommand::PauseInstance(handle, fade_tween) => {
				let instance_id = self.instances.get(&handle).unwrap();
				SequenceOutputCommand::PauseInstance(*instance_id, fade_tween)
			}
			SequenceOutputCommand::ResumeInstance(handle, fade_tween) => {
				let instance_id = self.instances.get(&handle).unwrap();
				SequenceOutputCommand::ResumeInstance(*instance_id, fade_tween)
			}
			SequenceOutputCommand::StopInstance(handle, fade_tween) => {
				let instance_id = self.instances.get(&handle).unwrap();
				SequenceOutputCommand::StopInstance(*instance_id, fade_tween)
			}
			SequenceOutputCommand::PauseInstancesOfSound(sound_id, fade_tween) => {
				SequenceOutputCommand::PauseInstancesOfSound(sound_id, fade_tween)
			}
			SequenceOutputCommand::ResumeInstancesOfSound(sound_id, fade_tween) => {
				SequenceOutputCommand::ResumeInstancesOfSound(sound_id, fade_tween)
			}
			SequenceOutputCommand::StopInstancesOfSound(sound_id, fade_tween) => {
				SequenceOutputCommand::StopInstancesOfSound(sound_id, fade_tween)
			}
			SequenceOutputCommand::SetMetronomeTempo(tempo) => {
				SequenceOutputCommand::SetMetronomeTempo(tempo)
			}
			SequenceOutputCommand::StartMetronome => SequenceOutputCommand::StartMetronome,
			SequenceOutputCommand::PauseMetronome => SequenceOutputCommand::PauseMetronome,
			SequenceOutputCommand::StopMetronome => SequenceOutputCommand::StopMetronome,
			SequenceOutputCommand::EmitCustomEvent(event) => {
				SequenceOutputCommand::EmitCustomEvent(event)
			}
		}
	}

	pub(crate) fn update(
		&mut self,
		dt: f64,
		metronome: &Metronome,
		output_command_queue: &mut Vec<SequenceOutputCommand<InstanceId, CustomEvent>>,
	) {
		loop {
			match self.state {
				SequenceState::Paused | SequenceState::Finished => {
					break;
				}
				_ => {
					if let Some(task) = self.tasks.get(self.position) {
						let task = *task;
						match task {
							SequenceTask::Wait(duration) => {
								if let Some(time) = self.wait_timer.as_mut() {
									let duration = duration.in_seconds(metronome.effective_tempo());
									*time -= dt / duration;
									if *time <= 0.0 {
										self.start_task(self.position + 1);
									}
									break;
								}
							}
							SequenceTask::WaitForInterval(interval) => {
								if metronome.interval_passed(interval) {
									self.start_task(self.position + 1);
								}
								break;
							}
							SequenceTask::RunCommand(command) => {
								if !self.muted {
									output_command_queue
										.push(self.transform_output_command(command));
								}
								self.start_task(self.position + 1);
							}
						}
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
