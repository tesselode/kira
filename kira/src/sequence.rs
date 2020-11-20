//! Provides an interface to script timed audio events.
//!
//! ## Creating and starting sequences
//!
//! To create a sequence, use `Sequence::new()` and then add
//! the actions you want to take in order:
//!
//! ```no_run
//! # use kira::{
//! # 	instance::InstanceSettings,
//! # 	manager::{AudioManager, AudioManagerSettings},
//! # 	sequence::Sequence,
//! # 	sound::{Sound, SoundMetadata, SoundSettings},
//! # 	Duration, Tempo,
//! # };
//! # let mut audio_manager = AudioManager::<()>::new(Default::default())?;
//! # let sound_id = audio_manager.add_sound(Sound::from_file("loop.ogg", Default::default())?)?;
//! let mut sequence = Sequence::new();
//! // play a sound
//! let instance_id = sequence.play_sound(sound_id, InstanceSettings::default());
//! // wait 2 seconds
//! sequence.wait(Duration::Seconds(2.0));
//! // stop the sound
//! sequence.stop_instance(instance_id, None);
//! # audio_manager.start_sequence(sequence)?;
//! # Ok::<(), kira::AudioError>(())
//! ```
//!
//! To start the sequence, use `AudioManager::start_sequence()`:
//!
//! ```no_run
//! # use kira::{
//! # 	instance::InstanceSettings,
//! # 	manager::{AudioManager, AudioManagerSettings},
//! # 	sequence::Sequence,
//! # 	sound::{Sound, SoundMetadata, SoundSettings},
//! # 	Duration, Tempo,
//! # };
//! # let mut audio_manager = AudioManager::<()>::new(Default::default())?;
//! # let sound_id = audio_manager.add_sound(Sound::from_file("loop.ogg", Default::default())?)?;
//! # let mut sequence = Sequence::new();
//! audio_manager.start_sequence(sequence)?;
//! # Ok::<(), kira::AudioError>(())
//! ```
//!
//! ## Timing events
//!
//! Sequences provide two ways of timing events:
//! - waiting for specific amounts of time
//! - waiting for a certain metronome interval
//!
//! This sequence will play a sound at the beginning of a measure
//! (assuming a measure is 4 beats long):
//!
//! ```no_run
//! # use kira::{
//! # 	instance::InstanceSettings,
//! # 	manager::{AudioManager, AudioManagerSettings},
//! # 	sequence::Sequence,
//! # 	sound::{Sound, SoundMetadata, SoundSettings},
//! # 	Duration, Tempo,
//! # };
//! # let mut audio_manager = AudioManager::<()>::new(Default::default())?;
//! # let sound_id = audio_manager.add_sound(Sound::from_file("loop.ogg", Default::default())?)?;
//! # let mut sequence = Sequence::new();
//! sequence.wait_for_interval(4.0);
//! sequence.play_sound(sound_id, InstanceSettings::default());
//! # audio_manager.start_sequence(sequence)?;
//! # Ok::<(), kira::AudioError>(())
//! ```
//!
//! Note that the metronome has to be running for the interval to work.
//!
//! ## Looping sequences
//!
//! You can create looping sequences by setting the loop start point. The
//! following example will wait for the start of a measure and then
//! play a sound every beat:
//!
//! ```no_run
//! # use kira::{
//! # 	instance::InstanceSettings,
//! # 	manager::{AudioManager, AudioManagerSettings},
//! # 	sequence::Sequence,
//! # 	sound::{Sound, SoundMetadata, SoundSettings},
//! # 	Duration, Tempo,
//! # };
//! # let mut audio_manager = AudioManager::<()>::new(Default::default())?;
//! # let sound_id = audio_manager.add_sound(Sound::from_file("loop.ogg", Default::default())?)?;
//! # let mut sequence = Sequence::new();
//! sequence.wait_for_interval(4.0);
//! sequence.start_loop();
//! // when the sequence finishes, it will loop back to this step
//! sequence.play_sound(sound_id, InstanceSettings::default());
//! sequence.wait(Duration::Beats(1.0));
//! # audio_manager.start_sequence(sequence)?;
//! # Ok::<(), kira::AudioError>(())
//! ```
//!
//! ## Custom events
//!
//! Sequences can emit custom events that you can receive on the main
//! thread, which is useful for syncing gameplay events to music events:
//!
//! ```no_run
//! # use kira::{
//! # 	instance::InstanceSettings,
//! # 	manager::{AudioManager, AudioManagerSettings},
//! # 	sequence::Sequence,
//! # 	sound::{Sound, SoundMetadata, SoundSettings},
//! # 	Duration, Tempo,
//! # };
//! # #[derive(Debug, Copy, Clone)]
//! # enum CustomEvent {
//! # 	Beat,
//! # }
//! # let mut audio_manager = AudioManager::<CustomEvent>::new(Default::default())?;
//! # let sound_id = audio_manager.add_sound(Sound::from_file("loop.ogg", Default::default())?)?;
//! # let mut sequence = Sequence::new();
//! sequence.wait_for_interval(4.0);
//! sequence.start_loop();
//! sequence.emit_custom_event(CustomEvent::Beat);
//! sequence.wait(Duration::Beats(1.0));
//! # audio_manager.start_sequence(sequence)?;
//! # Ok::<(), kira::AudioError>(())
//! ```
//!
//! To retrieve the events, use `AudioManager::events()`:
//!
//! ```no_run
//! # use kira::{
//! # 	instance::InstanceSettings,
//! # 	manager::{AudioManager, AudioManagerSettings},
//! # 	sequence::Sequence,
//! # 	sound::{Sound, SoundMetadata, SoundSettings},
//! # 	Duration, Tempo,
//! # };
//! #
//! # let mut audio_manager = AudioManager::<()>::new(Default::default())?;
//! for event in audio_manager.events() {
//! 	println!("{:?}", event);
//! }
//! # Ok::<(), kira::AudioError>(())
//! ```

use std::sync::atomic::{AtomicUsize, Ordering};

use crate::{
	instance::{InstanceId, InstanceSettings},
	metronome::Metronome,
	parameter::{ParameterId, Tween},
	sound::SoundId,
	AudioError, AudioResult, Duration, Tempo, Value,
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
	SetInstanceVolume(InstanceId, Value<f64>),
	SetInstancePitch(InstanceId, Value<f64>),
	SetInstancePanning(InstanceId, Value<f64>),
	PauseInstance(InstanceId, Option<Tween>),
	ResumeInstance(InstanceId, Option<Tween>),
	StopInstance(InstanceId, Option<Tween>),
	PauseInstancesOfSound(SoundId, Option<Tween>),
	ResumeInstancesOfSound(SoundId, Option<Tween>),
	StopInstancesOfSound(SoundId, Option<Tween>),
	PauseSequence(SequenceId),
	ResumeSequence(SequenceId),
	StopSequence(SequenceId),
	PauseInstancesOfSequence(SequenceId, Option<Tween>),
	ResumeInstancesOfSequence(SequenceId, Option<Tween>),
	StopInstancesOfSequence(SequenceId, Option<Tween>),
	SetMetronomeTempo(Value<Tempo>),
	StartMetronome,
	PauseMetronome,
	StopMetronome,
	SetParameter(ParameterId, f64, Option<Tween>),
	EmitCustomEvent(CustomEvent),
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
	/// Creates a new sequence.
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

	/// Marks the point the sequence will loop back to
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
	pub fn set_instance_volume(&mut self, id: InstanceId, volume: Value<f64>) {
		self.steps
			.push(SequenceOutputCommand::SetInstanceVolume(id, volume).into());
	}

	/// Adds a step to set the pitch of an instance.
	pub fn set_instance_pitch(&mut self, id: InstanceId, pitch: Value<f64>) {
		self.steps
			.push(SequenceOutputCommand::SetInstancePitch(id, pitch).into());
	}

	/// Adds a step to set the panning of an instance.
	pub fn set_instance_panning(&mut self, id: InstanceId, panning: Value<f64>) {
		self.steps
			.push(SequenceOutputCommand::SetInstancePanning(id, panning).into());
	}

	/// Adds a step to pause an instance.
	pub fn pause_instance(&mut self, id: InstanceId, fade_tween: Option<Tween>) {
		self.steps
			.push(SequenceOutputCommand::PauseInstance(id, fade_tween).into());
	}

	/// Adds a step to resume an instance.
	pub fn resume_instance(&mut self, id: InstanceId, fade_tween: Option<Tween>) {
		self.steps
			.push(SequenceOutputCommand::ResumeInstance(id, fade_tween).into());
	}

	/// Adds a step to stop an instance.
	pub fn stop_instance(&mut self, id: InstanceId, fade_tween: Option<Tween>) {
		self.steps
			.push(SequenceOutputCommand::StopInstance(id, fade_tween).into());
	}

	/// Adds a step to pause all instances of a sound.
	pub fn pause_instances_of_sound(&mut self, id: SoundId, fade_tween: Option<Tween>) {
		self.steps
			.push(SequenceOutputCommand::PauseInstancesOfSound(id, fade_tween).into());
	}

	/// Adds a step to resume all instances of a sound.
	pub fn resume_instances_of_sound(&mut self, id: SoundId, fade_tween: Option<Tween>) {
		self.steps
			.push(SequenceOutputCommand::ResumeInstancesOfSound(id, fade_tween).into());
	}

	/// Adds a step to stop all instances of a sound.
	pub fn stop_instances_of_sound(&mut self, id: SoundId, fade_tween: Option<Tween>) {
		self.steps
			.push(SequenceOutputCommand::StopInstancesOfSound(id, fade_tween).into());
	}

	/// Adds a step to pause a sequence and all instances played by it.
	pub fn pause_sequence_and_instances(&mut self, id: SequenceId, fade_tween: Option<Tween>) {
		self.steps
			.push(SequenceOutputCommand::PauseSequence(id).into());
		self.steps
			.push(SequenceOutputCommand::PauseInstancesOfSequence(id, fade_tween).into());
	}

	/// Adds a step to resume a sequence and all instances played by it.
	pub fn resume_sequence_and_instances(&mut self, id: SequenceId, fade_tween: Option<Tween>) {
		self.steps
			.push(SequenceOutputCommand::ResumeSequence(id).into());
		self.steps
			.push(SequenceOutputCommand::ResumeInstancesOfSequence(id, fade_tween).into());
	}

	/// Adds a step to stop a sequence and all instances played by it.
	pub fn stop_sequence_and_instances(&mut self, id: SequenceId, fade_tween: Option<Tween>) {
		self.steps
			.push(SequenceOutputCommand::StopSequence(id).into());
		self.steps
			.push(SequenceOutputCommand::StopInstancesOfSequence(id, fade_tween).into());
	}

	/// Adds a step to set the tempo of the metronome.
	pub fn set_metronome_tempo<T: Into<Value<Tempo>>>(&mut self, tempo: T) {
		self.steps
			.push(SequenceOutputCommand::SetMetronomeTempo(tempo.into()).into());
	}

	/// Adds a step to start the metronome.
	pub fn start_metronome(&mut self) {
		self.steps
			.push(SequenceOutputCommand::StartMetronome.into());
	}

	/// Adds a step to pause the metronome.
	pub fn pause_metronome(&mut self) {
		self.steps
			.push(SequenceOutputCommand::PauseMetronome.into());
	}

	/// Adds a step to stop the metronome.
	pub fn stop_metronome(&mut self) {
		self.steps.push(SequenceOutputCommand::StopMetronome.into());
	}

	/// Adds a step to set a parameter.
	pub fn set_parameter(&mut self, id: ParameterId, target: f64, tween: Option<Tween>) {
		self.steps
			.push(SequenceOutputCommand::SetParameter(id, target, tween).into());
	}

	/// Adds a step to emit a custom event.
	pub fn emit_custom_event(&mut self, event: CustomEvent) {
		self.steps
			.push(SequenceOutputCommand::EmitCustomEvent(event).into());
	}

	/// Makes sure nothing's wrong with the sequence that would make
	/// it unplayable. Currently, this only checks that the loop
	/// point isn't at the very end of the sequence.
	pub(crate) fn validate(&self) -> AudioResult<()> {
		if let Some(loop_point) = self.loop_point {
			if loop_point >= self.steps.len() {
				return Err(AudioError::InvalidSequenceLoopPoint);
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
									SequenceOutputCommand::SetInstanceVolume(id, _) => {
										if *id == old_instance_id {
											*id = new_instance_id;
										}
									}
									SequenceOutputCommand::SetInstancePitch(id, _) => {
										if *id == old_instance_id {
											*id = new_instance_id;
										}
									}
									SequenceOutputCommand::PauseInstance(id, _) => {
										if *id == old_instance_id {
											*id = new_instance_id;
										}
									}
									SequenceOutputCommand::ResumeInstance(id, _) => {
										if *id == old_instance_id {
											*id = new_instance_id;
										}
									}
									SequenceOutputCommand::StopInstance(id, _) => {
										if *id == old_instance_id {
											*id = new_instance_id;
										}
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
