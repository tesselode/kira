//! Provides an interface to script timed audio events.
//!
//! ## Creating and starting sequences
//!
//! To create a sequence, use `Sequence::new()` and then add
//! the actions you want to take in order:
//!
//! ```no_run
//! # use kira::{
//! # 	instance::{InstanceSettings, StopInstanceSettings},
//! # 	manager::{AudioManager, AudioManagerSettings},
//! # 	sequence::{Sequence, SequenceSettings, SequenceInstanceSettings},
//! # 	sound::Sound,
//! # 	Duration, Tempo,
//! # };
//! # let mut audio_manager = AudioManager::new(Default::default())?;
//! # let sound_id = audio_manager.add_sound(Sound::from_file("loop.ogg", Default::default())?)?;
//! let mut sequence = Sequence::<()>::new(SequenceSettings::default());
//! // play a sound
//! let instance_id = sequence.play(sound_id, InstanceSettings::default());
//! // wait 2 seconds
//! sequence.wait(Duration::Seconds(2.0));
//! // stop the sound
//! sequence.stop_instance(instance_id, StopInstanceSettings::default());
//! # audio_manager.start_sequence(sequence, SequenceInstanceSettings::default())?;
//! # Ok::<(), kira::AudioError>(())
//! ```
//!
//! To start the sequence, use `AudioManager::start_sequence()`:
//!
//! ```no_run
//! # use kira::{
//! # 	instance::InstanceSettings,
//! # 	manager::{AudioManager, AudioManagerSettings},
//! # 	sequence::{Sequence, SequenceSettings, SequenceInstanceSettings},
//! # 	sound::Sound,
//! # 	Duration, Tempo,
//! # };
//! # let mut audio_manager = AudioManager::new(Default::default())?;
//! # let sound_id = audio_manager.add_sound(Sound::from_file("loop.ogg", Default::default())?)?;
//! # let mut sequence = Sequence::<()>::new(SequenceSettings::default());
//! audio_manager.start_sequence(sequence, SequenceInstanceSettings::default())?;
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
//! # 	sequence::{Sequence, SequenceSettings, SequenceInstanceSettings},
//! # 	sound::Sound,
//! # 	Duration, Tempo,
//! # };
//! # let mut audio_manager = AudioManager::new(Default::default())?;
//! # let sound_id = audio_manager.add_sound(Sound::from_file("loop.ogg", Default::default())?)?;
//! # let mut sequence = Sequence::<()>::new(SequenceSettings::default());
//! sequence.wait_for_interval(4.0);
//! sequence.play(sound_id, InstanceSettings::default());
//! # audio_manager.start_sequence(sequence, SequenceInstanceSettings::default())?;
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
//! # 	sequence::{Sequence, SequenceSettings, SequenceInstanceSettings},
//! # 	sound::Sound,
//! # 	Duration, Tempo,
//! # };
//! # let mut audio_manager = AudioManager::new(Default::default())?;
//! # let sound_id = audio_manager.add_sound(Sound::from_file("loop.ogg", Default::default())?)?;
//! # let mut sequence = Sequence::<()>::new(SequenceSettings::default());
//! sequence.wait_for_interval(4.0);
//! sequence.start_loop();
//! // when the sequence finishes, it will loop back to this step
//! sequence.play(sound_id, InstanceSettings::default());
//! sequence.wait(Duration::Beats(1.0));
//! # audio_manager.start_sequence(sequence, SequenceInstanceSettings::default())?;
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
//! # 	sequence::{Sequence, SequenceSettings, SequenceInstanceSettings},
//! # 	sound::Sound,
//! # 	Duration, Tempo,
//! # };
//! #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
//! enum CustomEvent {
//! 	Beat,
//! }
//! # let mut audio_manager = AudioManager::new(Default::default())?;
//! # let sound_id = audio_manager.add_sound(Sound::from_file("loop.ogg", Default::default())?)?;
//! # let mut sequence = Sequence::<CustomEvent>::new(SequenceSettings::default());
//! sequence.wait_for_interval(4.0);
//! sequence.start_loop();
//! sequence.emit(CustomEvent::Beat);
//! sequence.wait(Duration::Beats(1.0));
//! let (id, mut event_receiver) = audio_manager.start_sequence(sequence, SequenceInstanceSettings::default())?;
//! # Ok::<(), kira::AudioError>(())
//! ```
//!
//! When you start a sequence, an [`EventReceiver`](EventReceiver)
//! is returned that you can pop events from:
//!
//! ```no_run
//! # use kira::{
//! # 	instance::InstanceSettings,
//! # 	manager::{AudioManager, AudioManagerSettings},
//! # 	sequence::{Sequence, SequenceSettings, SequenceInstanceSettings},
//! # 	sound::Sound,
//! # 	Duration, Tempo,
//! # };
//! #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
//! enum CustomEvent {
//! 	Beat,
//! }
//! # let mut audio_manager = AudioManager::new(Default::default())?;
//! # let sound_id = audio_manager.add_sound(Sound::from_file("loop.ogg", Default::default())?)?;
//! # let mut sequence = Sequence::<CustomEvent>::new(SequenceSettings::default());
//! # sequence.wait_for_interval(4.0);
//! # sequence.start_loop();
//! # sequence.emit(CustomEvent::Beat);
//! # sequence.wait(Duration::Beats(1.0));
//! # let (id, mut event_receiver) = audio_manager.start_sequence(sequence, SequenceInstanceSettings::default())?;
//! while let Some(event) = event_receiver.pop() {
//! 	println!("{:?}", event);
//! }
//! # Ok::<(), kira::AudioError>(())
//! ```

mod handle;
mod instance;

pub use handle::SequenceInstanceHandle;
pub(crate) use instance::SequenceInstance;
pub use instance::{SequenceInstanceId, SequenceInstanceState};

use indexmap::IndexSet;

use std::{hash::Hash, vec};

use crate::{
	command::sender::CommandSender,
	group::{groups::Groups, GroupId, GroupSet},
	instance::{
		InstanceId, InstanceSettings, PauseInstanceSettings, ResumeInstanceSettings,
		StopInstanceSettings,
	},
	metronome::MetronomeId,
	parameter::{ParameterId, Tween},
	playable::Playable,
	AudioError, AudioResult, Duration, Tempo, Value,
};

/// Settings for an instance of a [`Sequence`].
#[derive(Debug, Copy, Clone)]
pub struct SequenceInstanceSettings {
	/// The metronome this sequence should sync to.
	pub metronome: Option<MetronomeId>,
	/// How many events can be queued at a time.
	pub event_queue_capacity: usize,
}

impl Default for SequenceInstanceSettings {
	fn default() -> Self {
		Self {
			metronome: None,
			event_queue_capacity: 10,
		}
	}
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum SequenceOutputCommand {
	PlaySound(InstanceId, Playable, InstanceSettings),
	SetInstanceVolume(InstanceId, Value<f64>),
	SetInstancePitch(InstanceId, Value<f64>),
	SetInstancePanning(InstanceId, Value<f64>),
	PauseInstance(InstanceId, PauseInstanceSettings),
	ResumeInstance(InstanceId, ResumeInstanceSettings),
	StopInstance(InstanceId, StopInstanceSettings),
	PauseInstancesOf(Playable, PauseInstanceSettings),
	ResumeInstancesOf(Playable, ResumeInstanceSettings),
	StopInstancesOf(Playable, StopInstanceSettings),
	PauseSequence(SequenceInstanceId),
	ResumeSequence(SequenceInstanceId),
	StopSequence(SequenceInstanceId),
	PauseInstancesOfSequence(SequenceInstanceId, PauseInstanceSettings),
	ResumeInstancesOfSequence(SequenceInstanceId, ResumeInstanceSettings),
	StopInstancesOfSequence(SequenceInstanceId, StopInstanceSettings),
	SetMetronomeTempo(MetronomeId, Value<Tempo>),
	StartMetronome(MetronomeId),
	PauseMetronome(MetronomeId),
	StopMetronome(MetronomeId),
	SetParameter(ParameterId, f64, Option<Tween>),
}

#[derive(Debug, Clone)]
pub(crate) enum SequenceStep<CustomEvent: Clone + Eq + Hash> {
	Wait(Duration),
	WaitForInterval(f64),
	RunCommand(SequenceOutputCommand),
	PlayRandom(InstanceId, Vec<Playable>, InstanceSettings),
	EmitCustomEvent(CustomEvent),
}

impl<CustomEvent: Clone + Eq + Hash> From<SequenceOutputCommand> for SequenceStep<CustomEvent> {
	fn from(command: SequenceOutputCommand) -> Self {
		Self::RunCommand(command)
	}
}

/// Settings for a [`Sequence`].
#[derive(Debug, Clone)]
pub struct SequenceSettings {
	/// The groups this sequence will belong to.
	pub groups: GroupSet,
}

impl SequenceSettings {
	/// Creates a new `SequenceSettings` with the default settings.
	pub fn new() -> Self {
		Self::default()
	}

	/// Sets the groups this sequence will belong to.
	pub fn groups(self, groups: impl Into<GroupSet>) -> Self {
		Self {
			groups: groups.into(),
			..self
		}
	}
}

impl Default for SequenceSettings {
	fn default() -> Self {
		Self {
			groups: GroupSet::new(),
		}
	}
}

/// A series of steps to execute at certain times.
#[derive(Debug, Clone)]
pub struct Sequence<CustomEvent: Clone + Eq + Hash = ()> {
	steps: Vec<SequenceStep<CustomEvent>>,
	loop_point: Option<usize>,
	groups: GroupSet,
}

impl<CustomEvent: Clone + Eq + Hash> Sequence<CustomEvent> {
	/// Creates a new sequence.
	pub fn new(settings: SequenceSettings) -> Self {
		Self {
			steps: vec![],
			loop_point: None,
			groups: settings.groups,
		}
	}

	fn with_components(
		steps: Vec<SequenceStep<CustomEvent>>,
		loop_point: Option<usize>,
		groups: GroupSet,
	) -> Self {
		Self {
			steps,
			loop_point,
			groups,
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

	/// Adds a step to play a random sound or arrangement from a
	/// list of choices.
	pub fn play_random(
		&mut self,
		choices: Vec<Playable>,
		settings: InstanceSettings,
	) -> InstanceId {
		let id = InstanceId::new();
		self.steps
			.push(SequenceStep::PlayRandom(id, choices, settings).into());
		id
	}

	/// Adds a step to set the volume of an instance.
	pub fn set_instance_volume(&mut self, id: impl Into<InstanceId>, volume: Value<f64>) {
		self.steps
			.push(SequenceOutputCommand::SetInstanceVolume(id.into(), volume).into());
	}

	/// Adds a step to set the pitch of an instance.
	pub fn set_instance_pitch(&mut self, id: impl Into<InstanceId>, pitch: Value<f64>) {
		self.steps
			.push(SequenceOutputCommand::SetInstancePitch(id.into(), pitch).into());
	}

	/// Adds a step to set the panning of an instance.
	pub fn set_instance_panning(&mut self, id: impl Into<InstanceId>, panning: Value<f64>) {
		self.steps
			.push(SequenceOutputCommand::SetInstancePanning(id.into(), panning).into());
	}

	/// Adds a step to pause an instance.
	pub fn pause_instance(&mut self, id: impl Into<InstanceId>, settings: PauseInstanceSettings) {
		self.steps
			.push(SequenceOutputCommand::PauseInstance(id.into(), settings).into());
	}

	/// Adds a step to resume an instance.
	pub fn resume_instance(&mut self, id: impl Into<InstanceId>, settings: ResumeInstanceSettings) {
		self.steps
			.push(SequenceOutputCommand::ResumeInstance(id.into(), settings).into());
	}

	/// Adds a step to stop an instance.
	pub fn stop_instance(&mut self, id: impl Into<InstanceId>, settings: StopInstanceSettings) {
		self.steps
			.push(SequenceOutputCommand::StopInstance(id.into(), settings).into());
	}

	/// Adds a step to pause all instances of a sound or arrangement.
	pub fn pause_instances_of(
		&mut self,
		playable: impl Into<Playable>,
		settings: PauseInstanceSettings,
	) {
		self.steps
			.push(SequenceOutputCommand::PauseInstancesOf(playable.into(), settings).into());
	}

	/// Adds a step to resume all instances of a sound or arrangement.
	pub fn resume_instances_of(
		&mut self,
		playable: impl Into<Playable>,
		settings: ResumeInstanceSettings,
	) {
		self.steps
			.push(SequenceOutputCommand::ResumeInstancesOf(playable.into(), settings).into());
	}

	/// Adds a step to stop all instances of a sound or arrangement.
	pub fn stop_instances_of(
		&mut self,
		playable: impl Into<Playable>,
		settings: StopInstanceSettings,
	) {
		self.steps
			.push(SequenceOutputCommand::StopInstancesOf(playable.into(), settings).into());
	}

	/// Adds a step to pause a sequence.
	pub fn pause_sequence(&mut self, id: impl Into<SequenceInstanceId>) {
		self.steps
			.push(SequenceOutputCommand::PauseSequence(id.into()).into());
	}

	/// Adds a step to resume a sequence.
	pub fn resume_sequence(&mut self, id: impl Into<SequenceInstanceId>) {
		self.steps
			.push(SequenceOutputCommand::ResumeSequence(id.into()).into());
	}

	/// Adds a step to stop a sequence.
	pub fn stop_sequence(&mut self, id: impl Into<SequenceInstanceId>) {
		self.steps
			.push(SequenceOutputCommand::StopSequence(id.into()).into());
	}

	/// Adds a step to pause a sequence and all instances played by it.
	pub fn pause_sequence_and_instances(
		&mut self,
		id: impl Into<SequenceInstanceId>,
		settings: PauseInstanceSettings,
	) {
		let id: SequenceInstanceId = id.into();
		self.steps
			.push(SequenceOutputCommand::PauseSequence(id).into());
		self.steps
			.push(SequenceOutputCommand::PauseInstancesOfSequence(id, settings).into());
	}

	/// Adds a step to resume a sequence and all instances played by it.
	pub fn resume_sequence_and_instances(
		&mut self,
		id: impl Into<SequenceInstanceId>,
		settings: ResumeInstanceSettings,
	) {
		let id: SequenceInstanceId = id.into();
		self.steps
			.push(SequenceOutputCommand::ResumeSequence(id).into());
		self.steps
			.push(SequenceOutputCommand::ResumeInstancesOfSequence(id, settings).into());
	}

	/// Adds a step to stop a sequence and all instances played by it.
	pub fn stop_sequence_and_instances(
		&mut self,
		id: impl Into<SequenceInstanceId>,
		settings: StopInstanceSettings,
	) {
		let id: SequenceInstanceId = id.into();
		self.steps
			.push(SequenceOutputCommand::StopSequence(id).into());
		self.steps
			.push(SequenceOutputCommand::StopInstancesOfSequence(id, settings).into());
	}

	/// Adds a step to set the tempo of the metronome.
	pub fn set_metronome_tempo(
		&mut self,
		id: impl Into<MetronomeId>,
		tempo: impl Into<Value<Tempo>>,
	) {
		self.steps
			.push(SequenceOutputCommand::SetMetronomeTempo(id.into(), tempo.into()).into());
	}

	/// Adds a step to start the metronome.
	pub fn start_metronome(&mut self, id: impl Into<MetronomeId>) {
		self.steps
			.push(SequenceOutputCommand::StartMetronome(id.into()).into());
	}

	/// Adds a step to pause the metronome.
	pub fn pause_metronome(&mut self, id: impl Into<MetronomeId>) {
		self.steps
			.push(SequenceOutputCommand::PauseMetronome(id.into()).into());
	}

	/// Adds a step to stop the metronome.
	pub fn stop_metronome(&mut self, id: impl Into<MetronomeId>) {
		self.steps
			.push(SequenceOutputCommand::StopMetronome(id.into()).into());
	}

	/// Adds a step to set a parameter.
	pub fn set_parameter(&mut self, id: impl Into<ParameterId>, target: f64, tween: Option<Tween>) {
		self.steps
			.push(SequenceOutputCommand::SetParameter(id.into(), target, tween).into());
	}

	/// Adds a step to emit a custom event.
	pub fn emit(&mut self, event: CustomEvent) {
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

	/// Gets a set of all of the events this sequence can emit.
	fn all_events(&self) -> IndexSet<CustomEvent> {
		let mut events = IndexSet::new();
		for step in &self.steps {
			if let SequenceStep::EmitCustomEvent(event) = step {
				events.insert(event.clone());
			}
		}
		events
	}

	/// Converts this sequence into a sequence where the custom events
	/// are indices corresponding to an event. Returns both the sequence
	/// and a mapping of indices to events.
	fn into_raw_sequence(&self) -> (RawSequence, IndexSet<CustomEvent>) {
		let events = self.all_events();
		let raw_steps = self
			.steps
			.iter()
			.map(|step| match step {
				SequenceStep::Wait(duration) => SequenceStep::Wait(*duration),
				SequenceStep::WaitForInterval(interval) => SequenceStep::WaitForInterval(*interval),
				SequenceStep::RunCommand(command) => SequenceStep::RunCommand(*command),
				SequenceStep::PlayRandom(id, choices, settings) => {
					SequenceStep::PlayRandom(*id, choices.clone(), *settings)
				}
				SequenceStep::EmitCustomEvent(event) => {
					SequenceStep::EmitCustomEvent(events.get_index_of(event).unwrap())
				}
			})
			.collect();
		(
			Sequence::with_components(raw_steps, self.loop_point, self.groups.clone()),
			events,
		)
	}

	pub(crate) fn create_instance(
		&self,
		settings: SequenceInstanceSettings,
		id: SequenceInstanceId,
		command_sender: CommandSender,
	) -> (SequenceInstance, SequenceInstanceHandle<CustomEvent>) {
		let (raw_sequence, events) = self.into_raw_sequence();
		let (event_sender, event_receiver) = flume::bounded(settings.event_queue_capacity);
		let instance = SequenceInstance::new(raw_sequence, event_sender, settings.metronome);
		let handle = SequenceInstanceHandle::new(
			id,
			instance.public_state(),
			command_sender,
			event_receiver,
			events,
		);
		(instance, handle)
	}

	/// Returns if this sequence is in the group with the given ID.
	pub(crate) fn is_in_group(&self, id: GroupId, all_groups: &Groups) -> bool {
		self.groups.has_ancestor(id, all_groups)
	}
}

pub(crate) type RawSequence = Sequence<usize>;

impl RawSequence {
	fn convert_ids(steps: &mut Vec<SequenceStep<usize>>, old_id: InstanceId, new_id: InstanceId) {
		for step in steps {
			match step {
				SequenceStep::RunCommand(command) => match command {
					SequenceOutputCommand::PlaySound(id, _, _) => {
						if *id == old_id {
							*id = new_id;
						}
					}
					SequenceOutputCommand::SetInstanceVolume(id, _) => {
						if *id == old_id {
							*id = new_id;
						}
					}
					SequenceOutputCommand::SetInstancePitch(id, _) => {
						if *id == old_id {
							*id = new_id;
						}
					}
					SequenceOutputCommand::SetInstancePanning(id, _) => {
						if *id == old_id {
							*id = new_id;
						}
					}
					SequenceOutputCommand::PauseInstance(id, _) => {
						if *id == old_id {
							*id = new_id;
						}
					}
					SequenceOutputCommand::ResumeInstance(id, _) => {
						if *id == old_id {
							*id = new_id;
						}
					}
					SequenceOutputCommand::StopInstance(id, _) => {
						if *id == old_id {
							*id = new_id;
						}
					}
					_ => {}
				},
				SequenceStep::PlayRandom(id, _, _) => {
					if *id == old_id {
						*id = new_id;
					}
				}
				_ => {}
			}
		}
	}

	/// Assigns new instance IDs to each PlaySound command and updates
	/// other sequence commands to use the new instance ID. This allows
	/// the sequence to play sounds with fresh instance IDs on each loop
	/// while still correctly pausing instances, setting their parameters,
	/// etc.
	fn update_instance_ids(&mut self) {
		for i in 0..self.steps.len() {
			match &self.steps[i] {
				SequenceStep::RunCommand(command) => match command {
					SequenceOutputCommand::PlaySound(id, _, _) => {
						let old_id = *id;
						Self::convert_ids(&mut self.steps, old_id, InstanceId::new());
					}
					_ => {}
				},
				SequenceStep::PlayRandom(id, _, _) => {
					let old_id = *id;
					Self::convert_ids(&mut self.steps, old_id, InstanceId::new());
				}
				_ => {}
			}
		}
	}
}
