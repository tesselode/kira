use basedrop::Owned;

use crate::{
	arrangement::{Arrangement, ArrangementId},
	command::ResourceCommand,
	sound::{Sound, SoundId},
	static_container::index_map::StaticIndexMap,
	Frame,
};

use super::{Playable, PlayableId, PlayableMut};

pub(crate) struct Playables {
	sounds: StaticIndexMap<SoundId, Owned<Sound>>,
	arrangements: StaticIndexMap<ArrangementId, Owned<Arrangement>>,
}

impl Playables {
	pub fn new(sound_capacity: usize, arrangement_capacity: usize) -> Self {
		Self {
			sounds: StaticIndexMap::new(sound_capacity),
			arrangements: StaticIndexMap::new(arrangement_capacity),
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
				self.sounds.try_insert(sound.id(), sound).ok();
			}
			ResourceCommand::RemoveSound(id) => {
				self.sounds.remove(&id);
			}
			ResourceCommand::AddArrangement(arrangement) => {
				self.arrangements
					.try_insert(arrangement.id(), arrangement)
					.ok();
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
