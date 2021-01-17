use crate::{
	arrangement::ArrangementId, audio_stream::AudioStreamId, group::GroupId,
	metronome::MetronomeId, mixer::SubTrackId, parameter::ParameterId, sound::SoundId,
};

use indexmap::IndexSet;

use super::{
	error::{
		AddArrangementError, AddGroupError, AddMetronomeError, AddParameterError, AddSoundError,
		AddStreamError, AddTrackError, RemoveArrangementError, RemoveGroupError,
		RemoveMetronomeError, RemoveParameterError, RemoveSoundError, RemoveStreamError,
		RemoveTrackError,
	},
	AudioManagerSettings,
};

pub struct ActiveIds {
	pub active_sound_ids: IndexSet<SoundId>,
	pub active_arrangement_ids: IndexSet<ArrangementId>,
	pub active_parameter_ids: IndexSet<ParameterId>,
	pub active_track_ids: IndexSet<SubTrackId>,
	pub active_group_ids: IndexSet<GroupId>,
	pub active_metronome_ids: IndexSet<MetronomeId>,
	pub active_stream_ids: IndexSet<AudioStreamId>,
}

impl ActiveIds {
	pub fn new(settings: &AudioManagerSettings) -> Self {
		Self {
			active_sound_ids: IndexSet::with_capacity(settings.num_sounds),
			active_arrangement_ids: IndexSet::with_capacity(settings.num_arrangements),
			active_parameter_ids: IndexSet::with_capacity(settings.num_parameters),
			active_track_ids: IndexSet::with_capacity(settings.num_tracks),
			active_group_ids: IndexSet::with_capacity(settings.num_groups),
			active_metronome_ids: IndexSet::with_capacity(settings.num_metronomes),
			active_stream_ids: IndexSet::with_capacity(settings.num_streams),
		}
	}

	pub fn add_sound_id(&mut self, id: SoundId) -> Result<(), AddSoundError> {
		if self.active_sound_ids.len() >= self.active_sound_ids.capacity() {
			return Err(AddSoundError::SoundLimitReached);
		}
		self.active_sound_ids.insert(id);
		Ok(())
	}

	pub fn remove_sound_id(&mut self, id: SoundId) -> Result<(), RemoveSoundError> {
		if !self.active_sound_ids.remove(&id) {
			return Err(RemoveSoundError::NoSoundWithId(id));
		}
		Ok(())
	}

	pub fn add_arrangement_id(&mut self, id: ArrangementId) -> Result<(), AddArrangementError> {
		if self.active_arrangement_ids.len() >= self.active_arrangement_ids.capacity() {
			return Err(AddArrangementError::ArrangementLimitReached);
		}
		self.active_arrangement_ids.insert(id);
		Ok(())
	}

	pub fn remove_arrangement_id(
		&mut self,
		id: ArrangementId,
	) -> Result<(), RemoveArrangementError> {
		if !self.active_arrangement_ids.remove(&id) {
			return Err(RemoveArrangementError::NoArrangementWithId(id));
		}
		Ok(())
	}

	pub fn add_parameter_id(&mut self, id: ParameterId) -> Result<(), AddParameterError> {
		if self.active_parameter_ids.len() >= self.active_parameter_ids.capacity() {
			return Err(AddParameterError::ParameterLimitReached);
		}
		self.active_parameter_ids.insert(id);
		Ok(())
	}

	pub fn remove_parameter_id(&mut self, id: ParameterId) -> Result<(), RemoveParameterError> {
		if !self.active_parameter_ids.remove(&id) {
			return Err(RemoveParameterError::NoParameterWithId(id));
		}
		Ok(())
	}

	pub fn add_track_id(&mut self, id: SubTrackId) -> Result<(), AddTrackError> {
		if self.active_track_ids.len() >= self.active_track_ids.capacity() {
			return Err(AddTrackError::TrackLimitReached);
		}
		self.active_track_ids.insert(id);
		Ok(())
	}

	pub fn remove_track_id(&mut self, id: SubTrackId) -> Result<(), RemoveTrackError> {
		if !self.active_track_ids.remove(&id) {
			return Err(RemoveTrackError::NoTrackWithId(id));
		}
		Ok(())
	}

	pub fn add_group_id(&mut self, id: GroupId) -> Result<(), AddGroupError> {
		if self.active_group_ids.len() >= self.active_group_ids.capacity() {
			return Err(AddGroupError::GroupLimitReached);
		}
		self.active_group_ids.insert(id);
		Ok(())
	}

	pub fn remove_group_id(&mut self, id: GroupId) -> Result<(), RemoveGroupError> {
		if !self.active_group_ids.remove(&id) {
			return Err(RemoveGroupError::NoGroupWithId(id));
		}
		Ok(())
	}

	pub fn add_metronome_id(&mut self, id: MetronomeId) -> Result<(), AddMetronomeError> {
		if self.active_metronome_ids.len() >= self.active_metronome_ids.capacity() {
			return Err(AddMetronomeError::MetronomeLimitReached);
		}
		self.active_metronome_ids.insert(id);
		Ok(())
	}

	pub fn remove_metronome_id(&mut self, id: MetronomeId) -> Result<(), RemoveMetronomeError> {
		if !self.active_metronome_ids.remove(&id) {
			return Err(RemoveMetronomeError::NoMetronomeWithId(id));
		}
		Ok(())
	}

	pub fn add_stream_id(&mut self, id: AudioStreamId) -> Result<(), AddStreamError> {
		if self.active_stream_ids.len() >= self.active_stream_ids.capacity() {
			return Err(AddStreamError::StreamLimitReached);
		}
		self.active_stream_ids.insert(id);
		Ok(())
	}

	pub fn remove_stream_id(&mut self, id: AudioStreamId) -> Result<(), RemoveStreamError> {
		if !self.active_stream_ids.remove(&id) {
			return Err(RemoveStreamError::NoStreamWithId(id));
		}
		Ok(())
	}
}
