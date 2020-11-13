use std::sync::atomic::{AtomicUsize, Ordering};

use crate::{
	command::{InstanceCommand, MetronomeCommand, ParameterCommand},
	instance::{InstanceId, InstanceSettings},
	metronome::Metronome,
	sound::SoundId,
	ConductorError, ConductorResult, Duration, Tween, Value,
};

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
pub(crate) enum SequenceOutputCommand<CustomEvent: Copy> {
	PlaySound(InstanceId, SoundId, InstanceSettings),
	Instance(InstanceCommand),
	Metronome(MetronomeCommand),
	Parameter(ParameterCommand),
	EmitCustomEvent(CustomEvent),
}

impl<CustomEvent: Copy> From<InstanceCommand> for SequenceOutputCommand<CustomEvent> {
	fn from(command: InstanceCommand) -> Self {
		Self::Instance(command)
	}
}

impl<CustomEvent: Copy> From<MetronomeCommand> for SequenceOutputCommand<CustomEvent> {
	fn from(command: MetronomeCommand) -> Self {
		Self::Metronome(command)
	}
}

impl<CustomEvent: Copy> From<ParameterCommand> for SequenceOutputCommand<CustomEvent> {
	fn from(command: ParameterCommand) -> Self {
		Self::Parameter(command)
	}
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum SequenceStep<CustomEvent: Copy> {
	Wait(Duration),
	WaitForInterval(f64),
	RunCommand(SequenceOutputCommand<CustomEvent>),
}

impl<CustomEvent: Copy> From<SequenceOutputCommand<CustomEvent>> for SequenceStep<CustomEvent> {
	fn from(command: SequenceOutputCommand<CustomEvent>) -> Self {
		Self::RunCommand(command)
	}
}

impl<CustomEvent: Copy> From<InstanceCommand> for SequenceStep<CustomEvent> {
	fn from(command: InstanceCommand) -> Self {
		Self::RunCommand(SequenceOutputCommand::Instance(command))
	}
}

impl<CustomEvent: Copy> From<MetronomeCommand> for SequenceStep<CustomEvent> {
	fn from(command: MetronomeCommand) -> Self {
		Self::RunCommand(SequenceOutputCommand::Metronome(command))
	}
}

impl<CustomEvent: Copy> From<ParameterCommand> for SequenceStep<CustomEvent> {
	fn from(command: ParameterCommand) -> Self {
		Self::RunCommand(SequenceOutputCommand::Parameter(command))
	}
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum SequenceState {
	Playing,
	Paused,
	Finished,
}

/// A series of audio-related actions to take at specific times.
#[derive(Debug, Clone)]
pub struct Sequence<CustomEvent: Copy> {
	steps: Vec<SequenceStep<CustomEvent>>,
	loop_point: Option<usize>,
	state: SequenceState,
	position: usize,
	wait_timer: Option<f64>,
	muted: bool,
}

impl<CustomEvent: Copy> Sequence<CustomEvent> {
	pub fn new() -> Self {
		Self {
			steps: vec![],
			loop_point: None,
			state: SequenceState::Playing,
			position: 0,
			wait_timer: None,
			muted: false,
		}
	}

	/// Adds a step to wait for a certain length of time
	/// before moving to the next step.
	pub fn wait(&mut self, duration: Duration) {
		self.steps.push(SequenceStep::Wait(duration));
	}

	/// Adds a step to wait for a certain metronome interval
	/// (in beats) to be passed before moving to the next step.
	pub fn wait_for_interval(&mut self, interval: f64) {
		self.steps.push(SequenceStep::WaitForInterval(interval));
	}

	/// Marks the point where the sequence will loop back to
	/// after it finishes the last step.
	pub fn start_loop(&mut self) {
		self.loop_point = Some(self.steps.len())
	}

	/// Adds a step to play a sound.
	pub fn play_sound(&mut self, sound_id: SoundId, settings: InstanceSettings) -> InstanceId {
		let id = InstanceId::new();
		self.steps
			.push(SequenceOutputCommand::PlaySound(id, sound_id, settings).into());
		id
	}

	/// Adds a step to set the volume of an instance.
	pub fn set_instance_volume(&mut self, id: InstanceId, volume: Value) {
		self.steps
			.push(InstanceCommand::SetInstanceVolume(id, volume).into());
	}

	/// Adds a step to set the pitch of an instance.
	pub fn set_instance_pitch(&mut self, id: InstanceId, pitch: Value) {
		self.steps
			.push(InstanceCommand::SetInstancePitch(id, pitch).into());
	}

	/// Adds a step to pause an instance.
	pub fn pause_instance(&mut self, id: InstanceId, fade_tween: Option<Tween>) {
		self.steps
			.push(InstanceCommand::PauseInstance(id, fade_tween).into());
	}

	/// Adds a step to resume an instance.
	pub fn resume_instance(&mut self, id: InstanceId, fade_tween: Option<Tween>) {
		self.steps
			.push(InstanceCommand::ResumeInstance(id, fade_tween).into());
	}

	/// Adds a step to stop an instance.
	pub fn stop_instance(&mut self, id: InstanceId, fade_tween: Option<Tween>) {
		self.steps
			.push(InstanceCommand::StopInstance(id, fade_tween).into());
	}

	/// Makes sure nothing's wrong with the sequence that would make
	/// it unplayable. Currently, this only checks that the loop
	/// point isn't at the very end of the sequence.
	pub(crate) fn validate(&self) -> ConductorResult<()> {
		if let Some(loop_point) = self.loop_point {
			if loop_point >= self.steps.len() {
				return Err(ConductorError::InvalidSequenceLoopPoint);
			}
		}
		Ok(())
	}

	/// Assigns new instance IDs to each PlaySound command and updates
	/// other sequence commands to use the new instance ID. This allows
	/// the sequence to play sounds with fresh instance IDs on each loop
	/// while still correctly pausing instances, setting their parameters,
	/// etc.
	fn update_instance_ids(&mut self) {
		for i in 0..self.steps.len() {
			match self.steps[i] {
				SequenceStep::RunCommand(command) => match command {
					SequenceOutputCommand::PlaySound(old_instance_id, sound_id, settings) => {
						let new_instance_id = InstanceId::new();
						self.steps[i] =
							SequenceOutputCommand::PlaySound(new_instance_id, sound_id, settings)
								.into();
						for step in &mut self.steps {
							match step {
								SequenceStep::RunCommand(command) => match command {
									SequenceOutputCommand::Instance(command) => {
										command.swap_instance_id(old_instance_id, new_instance_id);
									}
									_ => {}
								},
								_ => {}
							}
						}
					}
					_ => {}
				},
				_ => {}
			}
		}
	}

	fn start_step(&mut self, index: usize) {
		if let Some(step) = self.steps.get(index) {
			self.position = index;
			if let SequenceStep::Wait(_) = step {
				self.wait_timer = Some(1.0);
			} else {
				self.wait_timer = None;
			}
		} else if let Some(loop_point) = self.loop_point {
			self.update_instance_ids();
			self.start_step(loop_point);
		} else {
			self.state = SequenceState::Finished;
		}
	}

	pub(crate) fn start(&mut self) {
		self.start_step(0);
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

	pub(crate) fn update(
		&mut self,
		dt: f64,
		metronome: &Metronome,
		output_command_queue: &mut Vec<SequenceOutputCommand<CustomEvent>>,
	) {
		loop {
			match self.state {
				SequenceState::Paused | SequenceState::Finished => {
					break;
				}
				_ => {
					if let Some(step) = self.steps.get(self.position) {
						match step {
							SequenceStep::Wait(duration) => {
								if let Some(time) = self.wait_timer.as_mut() {
									let duration = duration.in_seconds(metronome.effective_tempo());
									*time -= dt / duration;
									if *time <= 0.0 {
										self.start_step(self.position + 1);
									}
									break;
								}
							}
							SequenceStep::WaitForInterval(interval) => {
								if metronome.interval_passed(*interval) {
									self.start_step(self.position + 1);
								}
								break;
							}
							SequenceStep::RunCommand(command) => {
								if !self.muted {
									output_command_queue.push(*command);
								}
								self.start_step(self.position + 1);
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
