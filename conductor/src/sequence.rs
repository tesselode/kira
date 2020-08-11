/*!
Contains structs related to sequences.

Sequences are useful for scripting a series of audio-related actions
with precise timing. Sequence tasks can be synced to a metronome, which
is useful if you want something to happen to the beat of some music.

## Example

```
# use conductor::{
# 	manager::AudioManagerSettings,
# 	metronome::MetronomeSettings,
# 	sequence::{PlaySoundTaskSettings, Sequence},
# 	time::Time,
# 	AudioManager, Project,
# };
# use std::{error::Error, path::PathBuf};
#
# fn main() -> Result<(), Box<dyn Error>> {
# let mut project = Project::new();
# let sound_id = project.load_sound(&PathBuf::from("whatever.ogg"))?;
# let metronome_id = project.create_metronome(120.0, MetronomeSettings::default());
# let mut audio_manager = AudioManager::new(project, AudioManagerSettings::default())?;
// create a new sequence that uses a previously created metronome
let mut sequence = Sequence::new(metronome_id);
// let's define the steps for the sequence:
// 1. wait for the next beat
sequence.on_interval(1.0);
// 2. play a sound
let task = sequence.play_sound(sound_id, PlaySoundTaskSettings::default());
// 3. wait for 4 beats
sequence.wait(Time::Beats(4.0));
// 4. stop the sound
sequence.stop_instance(task, None);
// 5. go to step 2
sequence.go_to(1);
// now that we've defined the sequence, let's start it
audio_manager.start_sequence(sequence)?;
# Ok(())
# }
```
*/

use crate::{
	command::{Command, InstanceCommand},
	instance::{InstanceId, InstanceSettings},
	metronome::{Metronome, MetronomeId},
	sound::SoundId,
	time::Time,
	tween::Tween,
};
use std::{
	collections::HashMap,
	sync::atomic::{AtomicUsize, Ordering},
};

static NEXT_PLAY_SOUND_TASK_HANDLE_INDEX: AtomicUsize = AtomicUsize::new(0);

/// A handle to a "play sound" task in a sequence.
///
/// This can be used to pause or resume an instance in a
/// later task in the sequence.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct PlaySoundTaskHandle {
	index: usize,
}

impl PlaySoundTaskHandle {
	pub fn new() -> Self {
		let index = NEXT_PLAY_SOUND_TASK_HANDLE_INDEX.fetch_add(1, Ordering::Relaxed);
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

/// Settings for playing a sound in a sequence.
///
/// This is almost identical to `InstanceSettings`, except
/// the position of the instance can be specified in beats
/// or seconds.
#[derive(Debug, Clone)]
pub struct PlaySoundTaskSettings {
	/// The volume to play the sound with.
	pub volume: f32,
	/// The pitch to play the sound with (as a factor of the original pitch).
	pub pitch: f32,
	/// The position to start the sound at.
	pub position: Time,
}

impl PlaySoundTaskSettings {
	fn into_instance_settings(&self, tempo: f32) -> InstanceSettings {
		InstanceSettings {
			volume: self.volume,
			pitch: self.pitch,
			position: self.position.in_seconds(tempo),
			fade_in_duration: None,
		}
	}
}

impl Default for PlaySoundTaskSettings {
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
enum SequenceTask {
	Wait(Time),
	WaitForInterval(f32),
	GoTo(usize),
	PlaySound(SoundId, PlaySoundTaskHandle, PlaySoundTaskSettings),
	PauseInstance(PlaySoundTaskHandle, Option<Tween<Time>>),
	ResumeInstance(PlaySoundTaskHandle, Option<Tween<Time>>),
	StopInstance(PlaySoundTaskHandle, Option<Tween<Time>>),
}

/**
A series of audio-related tasks to perform.

A sequence performs tasks in order. A task can be:
- A command for the `AudioManager`, like playing a sound,
or changing the volume of an instance
- Waiting for a duration of time or for a specific moment in time
- Returning to a previous task (important for creating looping sequences)
*/
#[derive(Debug, Clone)]
pub struct Sequence {
	pub metronome_id: MetronomeId,
	tasks: Vec<SequenceTask>,
	state: SequenceState,
	wait_timer: Option<f32>,
	instances: HashMap<PlaySoundTaskHandle, InstanceId>,
}

impl Sequence {
	/// Creates a new sequence synced to the given metronome.
	pub fn new(metronome_id: MetronomeId) -> Self {
		Self {
			metronome_id,
			tasks: vec![],
			state: SequenceState::Idle,
			wait_timer: None,
			instances: HashMap::new(),
		}
	}

	/// Adds a task to wait for a duration of time before
	/// moving to the next task.
	pub fn wait(&mut self, time: Time) {
		self.tasks.push(SequenceTask::Wait(time));
	}

	/// Adds a task to wait for a certain metronome interval (in beats)
	/// before moving to the next task.
	pub fn on_interval(&mut self, interval: f32) {
		self.tasks.push(SequenceTask::WaitForInterval(interval));
	}

	/// Adds a task to play a sound.
	pub fn play_sound(
		&mut self,
		sound_id: SoundId,
		settings: PlaySoundTaskSettings,
	) -> PlaySoundTaskHandle {
		self.instances.reserve(1);
		let sequence_instance_handle = PlaySoundTaskHandle::new();
		self.tasks.push(SequenceTask::PlaySound(
			sound_id,
			sequence_instance_handle,
			settings,
		));
		sequence_instance_handle
	}

	/// Adds a task to pause an instance of a sound created by a play sound task.
	pub fn pause_instance(&mut self, handle: PlaySoundTaskHandle, fade_tween: Option<Tween<Time>>) {
		self.tasks
			.push(SequenceTask::PauseInstance(handle, fade_tween));
	}

	/// Adds a task to resume an instance of a sound created by a play sound task.
	pub fn resume_instance(
		&mut self,
		handle: PlaySoundTaskHandle,
		fade_tween: Option<Tween<Time>>,
	) {
		self.tasks
			.push(SequenceTask::ResumeInstance(handle, fade_tween));
	}

	/// Adds a task to stop an instance of a sound created by a play sound task.
	pub fn stop_instance(&mut self, handle: PlaySoundTaskHandle, fade_tween: Option<Tween<Time>>) {
		self.tasks
			.push(SequenceTask::StopInstance(handle, fade_tween));
	}

	/// Adds a task to jump to the nth task in the sequence.
	pub fn go_to(&mut self, index: usize) {
		self.tasks.push(SequenceTask::GoTo(index));
	}

	fn go_to_command(
		&mut self,
		index: usize,
		metronome: &Metronome,
		command_queue: &mut Vec<Command>,
	) {
		if index >= self.tasks.len() {
			self.state = SequenceState::Finished;
			return;
		}
		self.state = SequenceState::Playing(index);
		if let Some(task) = self.tasks.get(index) {
			let command = task.clone();
			if let SequenceTask::Wait(_) = command {
				self.wait_timer = Some(1.0);
			} else {
				self.wait_timer = None;
			}
			match command {
				SequenceTask::GoTo(index) => {
					self.go_to_command(index, metronome, command_queue);
				}
				SequenceTask::PlaySound(sound_id, sequence_instance_handle, settings) => {
					let instance_id = InstanceId::new();
					self.instances.insert(sequence_instance_handle, instance_id);
					command_queue.push(Command::Instance(InstanceCommand::PlaySound(
						sound_id,
						instance_id,
						settings.into_instance_settings(metronome.tempo),
					)));
					self.go_to_command(index + 1, metronome, command_queue);
				}
				SequenceTask::PauseInstance(handle, fade_tween) => {
					if let Some(instance_id) = self.instances.get(&handle) {
						let fade_tween = match fade_tween {
							Some(tween) => Some(tween.in_seconds(metronome.tempo)),
							None => None,
						};
						command_queue.push(Command::Instance(InstanceCommand::PauseInstance(
							*instance_id,
							fade_tween,
						)))
					}
					self.go_to_command(index + 1, metronome, command_queue);
				}
				SequenceTask::ResumeInstance(handle, fade_tween) => {
					if let Some(instance_id) = self.instances.get(&handle) {
						let fade_tween = match fade_tween {
							Some(tween) => Some(tween.in_seconds(metronome.tempo)),
							None => None,
						};
						command_queue.push(Command::Instance(InstanceCommand::ResumeInstance(
							*instance_id,
							fade_tween,
						)))
					}
					self.go_to_command(index + 1, metronome, command_queue);
				}
				SequenceTask::StopInstance(handle, fade_tween) => {
					if let Some(instance_id) = self.instances.get(&handle) {
						let fade_tween = match fade_tween {
							Some(tween) => Some(tween.in_seconds(metronome.tempo)),
							None => None,
						};
						command_queue.push(Command::Instance(InstanceCommand::StopInstance(
							*instance_id,
							fade_tween,
						)))
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
			if let Some(task) = self.tasks.get(index) {
				match task {
					SequenceTask::Wait(time) => {
						let time = time.in_seconds(metronome.effective_tempo());
						if let Some(wait_timer) = self.wait_timer.as_mut() {
							*wait_timer -= dt / time;
							if *wait_timer <= 0.0 {
								self.go_to_command(index + 1, metronome, command_queue);
							}
						}
					}
					SequenceTask::WaitForInterval(interval) => {
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
