use crate::{
	command::{sender::CommandSender, InstanceCommand},
	instance::{
		Instance, InstanceHandle, InstanceId, InstanceSettings, PauseInstanceSettings,
		ResumeInstanceSettings, StopInstanceSettings,
	},
	mixer::TrackIndex,
	AudioResult,
};

use super::{Sound, SoundId};

#[derive(Clone)]
pub struct SoundHandle {
	id: SoundId,
	duration: f64,
	default_track: TrackIndex,
	semantic_duration: Option<f64>,
	default_loop_start: Option<f64>,
	command_sender: CommandSender,
}

impl SoundHandle {
	pub(crate) fn new(id: SoundId, sound: &Sound, command_sender: CommandSender) -> Self {
		Self {
			id,
			duration: sound.duration(),
			default_track: sound.default_track(),
			semantic_duration: sound.semantic_duration(),
			default_loop_start: sound.default_loop_start(),
			command_sender,
		}
	}

	pub fn id(&self) -> SoundId {
		self.id
	}

	pub fn duration(&self) -> f64 {
		self.duration
	}

	pub fn default_track(&self) -> TrackIndex {
		self.default_track
	}

	pub fn semantic_duration(&self) -> Option<f64> {
		self.semantic_duration
	}

	pub fn default_loop_start(&self) -> Option<f64> {
		self.default_loop_start
	}

	pub fn play(&mut self, settings: InstanceSettings) -> AudioResult<InstanceHandle> {
		let instance_id = InstanceId::new();
		let instance = Instance::new(self.id.into(), None, settings);
		let handle = InstanceHandle::new(
			instance_id,
			instance.public_state(),
			self.command_sender.clone(),
		);
		self.command_sender
			.push(InstanceCommand::Play(instance_id, instance).into())
			.map(|()| handle)
	}

	pub fn pause(&mut self, settings: PauseInstanceSettings) -> AudioResult<()> {
		self.command_sender
			.push(InstanceCommand::PauseInstancesOf(self.id.into(), settings).into())
	}

	pub fn resume(&mut self, settings: ResumeInstanceSettings) -> AudioResult<()> {
		self.command_sender
			.push(InstanceCommand::ResumeInstancesOf(self.id.into(), settings).into())
	}

	pub fn stop(&mut self, settings: StopInstanceSettings) -> AudioResult<()> {
		self.command_sender
			.push(InstanceCommand::StopInstancesOf(self.id.into(), settings).into())
	}
}
