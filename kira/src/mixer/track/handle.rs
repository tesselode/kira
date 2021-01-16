use flume::Sender;

use crate::{
	audio_stream::{AudioStream, AudioStreamId},
	command::{Command, MixerCommand, StreamCommand},
	mixer::effect::{Effect, EffectHandle, EffectId, EffectSettings},
	AudioError, AudioResult,
};

use super::TrackIndex;

pub struct TrackHandle {
	index: TrackIndex,
	command_sender: Sender<Command>,
}

impl TrackHandle {
	pub(crate) fn new(index: TrackIndex, command_sender: Sender<Command>) -> Self {
		Self {
			index,
			command_sender,
		}
	}

	pub fn index(&self) -> TrackIndex {
		self.index
	}

	pub fn add_effect(
		&mut self,
		effect: impl Effect + 'static,
		settings: EffectSettings,
	) -> AudioResult<EffectHandle> {
		let handle = EffectHandle::new(self.index, &settings, self.command_sender.clone());
		self.command_sender
			.send(MixerCommand::AddEffect(self.index, Box::new(effect), settings).into())
			.map_err(|_| AudioError::BackendDisconnected)?;
		Ok(handle)
	}

	pub fn remove_effect(&mut self, id: impl Into<EffectId>) -> AudioResult<()> {
		self.command_sender
			.send(MixerCommand::RemoveEffect(self.index, id.into()).into())
			.map_err(|_| AudioError::BackendDisconnected)
	}

	pub fn add_stream(&mut self, stream: impl AudioStream) -> AudioResult<AudioStreamId> {
		let stream_id = AudioStreamId::new();
		self.command_sender
			.send(StreamCommand::AddStream(stream_id, self.index(), Box::new(stream)).into())
			.map_err(|_| AudioError::BackendDisconnected)
			.map(|()| stream_id)
	}

	pub fn remove_stream(&mut self, id: AudioStreamId) -> AudioResult<()> {
		self.command_sender
			.send(StreamCommand::RemoveStream(id).into())
			.map_err(|_| AudioError::BackendDisconnected)
	}
}
