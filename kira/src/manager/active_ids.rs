use crate::{
	arrangement::ArrangementId,
	error::{AudioError, AudioResult},
	group::GroupId,
	metronome::MetronomeId,
	mixer::SubTrackId,
	parameter::ParameterId,
	sound::SoundId,
};

use indexmap::IndexSet;

use super::AudioManagerSettings;

pub struct ActiveIds {
	pub active_sound_ids: IndexSet<SoundId>,
	pub active_arrangement_ids: IndexSet<ArrangementId>,
	pub active_parameter_ids: IndexSet<ParameterId>,
	pub active_track_ids: IndexSet<SubTrackId>,
	pub active_group_ids: IndexSet<GroupId>,
	pub active_metronome_ids: IndexSet<MetronomeId>,
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
		}
	}

	pub fn add_sound_id(&mut self, id: SoundId) -> AudioResult<()> {
		if self.active_sound_ids.len() >= self.active_sound_ids.capacity() {
			return Err(AudioError::SoundLimitReached);
		}
		self.active_sound_ids.insert(id);
		Ok(())
	}

	pub fn remove_sound_id(&mut self, id: SoundId) -> AudioResult<()> {
		if !self.active_sound_ids.remove(&id) {
			return Err(AudioError::NoSoundWithId(id));
		}
		Ok(())
	}

	pub fn add_arrangement_id(&mut self, id: ArrangementId) -> AudioResult<()> {
		if self.active_arrangement_ids.len() >= self.active_arrangement_ids.capacity() {
			return Err(AudioError::ArrangementLimitReached);
		}
		self.active_arrangement_ids.insert(id);
		Ok(())
	}

	pub fn remove_arrangement_id(&mut self, id: ArrangementId) -> AudioResult<()> {
		if !self.active_arrangement_ids.remove(&id) {
			return Err(AudioError::NoArrangementWithId(id));
		}
		Ok(())
	}

	pub fn add_parameter_id(&mut self, id: ParameterId) -> AudioResult<()> {
		if self.active_parameter_ids.len() >= self.active_parameter_ids.capacity() {
			return Err(AudioError::ParameterLimitReached);
		}
		self.active_parameter_ids.insert(id);
		Ok(())
	}

	pub fn remove_parameter_id(&mut self, id: ParameterId) -> AudioResult<()> {
		if !self.active_parameter_ids.remove(&id) {
			return Err(AudioError::NoParameterWithId(id));
		}
		Ok(())
	}

	pub fn add_track_id(&mut self, id: SubTrackId) -> AudioResult<()> {
		if self.active_track_ids.len() >= self.active_track_ids.capacity() {
			return Err(AudioError::TrackLimitReached);
		}
		self.active_track_ids.insert(id);
		Ok(())
	}

	pub fn remove_track_id(&mut self, id: SubTrackId) -> AudioResult<()> {
		if !self.active_track_ids.remove(&id) {
			return Err(AudioError::NoTrackWithId(id));
		}
		Ok(())
	}

	pub fn add_group_id(&mut self, id: GroupId) -> AudioResult<()> {
		if self.active_group_ids.len() >= self.active_group_ids.capacity() {
			return Err(AudioError::GroupLimitReached);
		}
		self.active_group_ids.insert(id);
		Ok(())
	}

	pub fn remove_group_id(&mut self, id: GroupId) -> AudioResult<()> {
		if !self.active_group_ids.remove(&id) {
			return Err(AudioError::NoGroupWithId(id));
		}
		Ok(())
	}

	pub fn add_metronome_id(&mut self, id: MetronomeId) -> AudioResult<()> {
		if self.active_metronome_ids.len() >= self.active_metronome_ids.capacity() {
			return Err(AudioError::MetronomeLimitReached);
		}
		self.active_metronome_ids.insert(id);
		Ok(())
	}

	pub fn remove_metronome_id(&mut self, id: MetronomeId) -> AudioResult<()> {
		if !self.active_metronome_ids.remove(&id) {
			return Err(AudioError::NoMetronomeWithId(id));
		}
		Ok(())
	}
}
