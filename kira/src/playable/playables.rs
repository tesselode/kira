use flume::Sender;
use indexmap::IndexMap;

use crate::{
	arrangement::{Arrangement, ArrangementId},
	command::ResourceCommand,
	resource::Resource,
	sound::{Sound, SoundId},
	Frame,
};

use super::{Playable, PlayableId, PlayableMut};

pub(crate) struct Playables {
	sounds: IndexMap<SoundId, Sound>,
	arrangements: IndexMap<ArrangementId, Arrangement>,
}

impl Playables {
	pub fn new(sound_capacity: usize, arrangement_capacity: usize) -> Self {
		Self {
			sounds: IndexMap::with_capacity(sound_capacity),
			arrangements: IndexMap::with_capacity(arrangement_capacity),
		}
	}

	pub fn sound(&self, id: SoundId) -> Option<&Sound> {
		self.sounds.get(&id)
	}

	pub fn sound_mut(&mut self, id: SoundId) -> Option<&mut Sound> {
		self.sounds.get_mut(&id)
	}

	pub fn arrangement(&self, id: ArrangementId) -> Option<&Arrangement> {
		self.arrangements.get(&id)
	}

	pub fn arrangement_mut(&mut self, id: ArrangementId) -> Option<&mut Arrangement> {
		self.arrangements.get_mut(&id)
	}

	pub fn playable(&self, id: PlayableId) -> Option<Playable> {
		match id {
			PlayableId::Sound(id) => self.sound(id).map(Playable::Sound),
			PlayableId::Arrangement(id) => self.arrangement(id).map(Playable::Arrangement),
		}
	}

	pub fn playable_mut(&mut self, id: PlayableId) -> Option<PlayableMut> {
		match id {
			PlayableId::Sound(id) => self.sound_mut(id).map(PlayableMut::Sound),
			PlayableId::Arrangement(id) => self.arrangement_mut(id).map(PlayableMut::Arrangement),
		}
	}

	pub fn frame_at_position(&self, id: PlayableId, position: f64) -> Option<Frame> {
		match id {
			PlayableId::Sound(id) => self
				.sound(id)
				.map(|sound| sound.get_frame_at_position(position)),
			PlayableId::Arrangement(id) => self
				.arrangement(id)
				.map(|arrangement| arrangement.get_frame_at_position(position, &self.sounds)),
		}
	}

	pub fn run_command(&mut self, command: ResourceCommand, unloader: &mut Sender<Resource>) {
		match command {
			ResourceCommand::AddSound(sound) => {
				if let Some(sound) = self.sounds.insert(sound.id(), sound) {
					unloader.try_send(Resource::Sound(sound)).ok();
				}
			}
			ResourceCommand::RemoveSound(id) => {
				if let Some(sound) = self.sounds.remove(&id) {
					unloader.try_send(Resource::Sound(sound)).ok();
				}
			}
			ResourceCommand::AddArrangement(arrangement) => {
				if let Some(arrangement) = self.arrangements.insert(arrangement.id(), arrangement) {
					unloader.try_send(Resource::Arrangement(arrangement)).ok();
				}
			}
			ResourceCommand::RemoveArrangement(id) => {
				if let Some(arrangement) = self.arrangements.remove(&id) {
					unloader.try_send(Resource::Arrangement(arrangement)).ok();
				}
			}
		}
	}

	pub fn update(&mut self, dt: f64) {
		for (_, sound) in &mut self.sounds {
			sound.update_cooldown(dt);
		}
		for (_, arrangement) in &mut self.arrangements {
			arrangement.update_cooldown(dt);
		}
	}
}
