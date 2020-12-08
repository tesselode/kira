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
//! # 	sound::Sound,
//! # 	Duration, Tempo,
//! # };
//! # let mut audio_manager = AudioManager::<()>::new(Default::default())?;
//! # let sound_id = audio_manager.add_sound(Sound::from_file("loop.ogg", Default::default())?)?;
//! let mut sequence = Sequence::new();
//! // play a sound
//! let instance_id = sequence.play(sound_id, InstanceSettings::default());
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
//! # 	sound::Sound,
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
//! # 	sound::Sound,
//! # 	Duration, Tempo,
//! # };
//! # let mut audio_manager = AudioManager::<()>::new(Default::default())?;
//! # let sound_id = audio_manager.add_sound(Sound::from_file("loop.ogg", Default::default())?)?;
//! # let mut sequence = Sequence::new();
//! sequence.wait_for_interval(4.0);
//! sequence.play(sound_id, InstanceSettings::default());
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
//! # 	sound::Sound,
//! # 	Duration, Tempo,
//! # };
//! # let mut audio_manager = AudioManager::<()>::new(Default::default())?;
//! # let sound_id = audio_manager.add_sound(Sound::from_file("loop.ogg", Default::default())?)?;
//! # let mut sequence = Sequence::new();
//! sequence.wait_for_interval(4.0);
//! sequence.start_loop();
//! // when the sequence finishes, it will loop back to this step
//! sequence.play(sound_id, InstanceSettings::default());
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
//! # 	sound::Sound,
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
//! To retrieve the events, use `AudioManager::pop_event()`:
//!
//! ```no_run
//! # use kira::{
//! # 	instance::InstanceSettings,
//! # 	manager::{AudioManager, AudioManagerSettings},
//! # 	sequence::Sequence,
//! # 	sound::Sound,
//! # 	Duration, Tempo,
//! # };
//! #
//! # let mut audio_manager = AudioManager::<()>::new(Default::default())?;
//! while let Some(event) = audio_manager.pop_event() {
//! 	println!("{:?}", event);
//! }
//! # Ok::<(), kira::AudioError>(())
//! ```

mod instance;

pub(crate) use instance::SequenceInstance;

use std::sync::atomic::{AtomicUsize, Ordering};

use crate::{
	instance::{InstanceId, InstanceSettings},
	parameter::{ParameterId, Tween},
	playable::Playable,
	AudioError, AudioResult, Duration, Tempo, Value,
};

static NEXT_SEQUENCE_INDEX: AtomicUsize = AtomicUsize::new(0);

/// A unique identifier for a [`Sequence`](crate::sequence::Sequence).
///
/// You cannot create this manually - a sequence ID is returned
/// when you start a sequence with an [`AudioManager`](crate::manager::AudioManager).
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
pub(crate) enum SequenceOutputCommand {
	PlaySound(InstanceId, Playable, InstanceSettings),
	SetInstanceVolume(InstanceId, Value<f64>),
	SetInstancePitch(InstanceId, Value<f64>),
	SetInstancePanning(InstanceId, Value<f64>),
	PauseInstance(InstanceId, Option<Tween>),
	ResumeInstance(InstanceId, Option<Tween>),
	StopInstance(InstanceId, Option<Tween>),
	PauseInstancesOf(Playable, Option<Tween>),
	ResumeInstancesOf(Playable, Option<Tween>),
	StopInstancesOf(Playable, Option<Tween>),
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
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum SequenceStep<CustomEvent: Copy> {
	Wait(Duration),
	WaitForInterval(f64),
	RunCommand(SequenceOutputCommand),
	EmitCustomEvent(CustomEvent),
}

impl<CustomEvent: Copy> From<SequenceOutputCommand> for SequenceStep<CustomEvent> {
	fn from(command: SequenceOutputCommand) -> Self {
		Self::RunCommand(command)
	}
}

#[derive(Debug, Clone)]
pub struct Sequence<CustomEvent: Copy> {
	steps: Vec<SequenceStep<CustomEvent>>,
	loop_point: Option<usize>,
}

impl<CustomEvent: Copy> Sequence<CustomEvent> {
	/// Creates a new sequence.
	pub fn new() -> Self {
		Self {
			steps: vec![],
			loop_point: None,
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

	/// Adds a step to play a sound or arrangement.
	pub fn play<P: Into<Playable>>(
		&mut self,
		playable: P,
		settings: InstanceSettings,
	) -> InstanceId {
		let id = InstanceId::new();
		self.steps
			.push(SequenceOutputCommand::PlaySound(id, playable.into(), settings).into());
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

	/// Adds a step to pause all instances of a sound or arrangement.
	pub fn pause_instances_of(&mut self, playable: Playable, fade_tween: Option<Tween>) {
		self.steps
			.push(SequenceOutputCommand::PauseInstancesOf(playable, fade_tween).into());
	}

	/// Adds a step to resume all instances of a sound or arrangement.
	pub fn resume_instances_of(&mut self, playable: Playable, fade_tween: Option<Tween>) {
		self.steps
			.push(SequenceOutputCommand::ResumeInstancesOf(playable, fade_tween).into());
	}

	/// Adds a step to stop all instances of a sound or arrangement.
	pub fn stop_instances_of(&mut self, playable: Playable, fade_tween: Option<Tween>) {
		self.steps
			.push(SequenceOutputCommand::StopInstancesOf(playable, fade_tween).into());
	}

	/// Adds a step to pause a sequence.
	pub fn pause_sequence(&mut self, id: SequenceId) {
		self.steps
			.push(SequenceOutputCommand::PauseSequence(id).into());
	}

	/// Adds a step to resume a sequence.
	pub fn resume_sequence(&mut self, id: SequenceId) {
		self.steps
			.push(SequenceOutputCommand::ResumeSequence(id).into());
	}

	/// Adds a step to stop a sequence.
	pub fn stop_sequence(&mut self, id: SequenceId) {
		self.steps
			.push(SequenceOutputCommand::StopSequence(id).into());
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
		self.steps.push(SequenceStep::EmitCustomEvent(event));
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
}
