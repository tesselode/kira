use basedrop::Owned;
use indexmap::IndexMap;

use crate::{
	arrangement::{Arrangement, ArrangementId},
	command::ResourceCommand,
	sound::{Sound, SoundId},
	Frame,
};

use super::{Playable, PlayableId, PlayableMut};

pub(crate) struct Playables {
	sounds: IndexMap<SoundId, Owned<Sound>>,
	arrangements: IndexMap<ArrangementId, Owned<Arrangement>>,
}

impl Playables {
	pub fn new(sound_capacity: usize, arrangement_capacity: usize) -> Self {
		Self {
			sounds: IndexMap::with_capacity(sound_capacity),
			arrangements: IndexMap::with_capacity(arrangement_capacity),
		}
	}

	pub fn sound(&self, id: SoundId) -> Option<&Owned<Sound>> {
		self.sounds.get(&id)
	}

	pub fn sound_mut(&mut self, id: SoundId) -> Option<&mut Owned<Sound>> {
		self.sounds.get_mut(&id)
	}

	pub fn arrangement(&self, id: ArrangementId) -> Option<&Owned<Arrangement>> {
		self.arrangements.get(&id)
	}

	pub fn arrangement_mut(&mut self, id: ArrangementId) -> Option<&mut Owned<Arrangement>> {
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

	pub fn run_command(&mut self, command: ResourceCommand) {
		match command {
			ResourceCommand::AddSound(sound) => {
				self.sounds.insert(sound.id(), sound);
			}
			ResourceCommand::RemoveSound(id) => {
				self.sounds.remove(&id);
			}
			ResourceCommand::AddArrangement(arrangement) => {
				self.arrangements.insert(arrangement.id(), arrangement);
			}
			ResourceCommand::RemoveArrangement(id) => {
				self.arrangements.remove(&id);
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
