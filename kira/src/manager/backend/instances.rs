use crate::{
	arrangement::{Arrangement, ArrangementId},
	command::InstanceCommand,
	group::groups::Groups,
	instance::{Instance, InstanceId, StopInstanceSettings},
	parameter::Parameters,
	playable::Playable,
	sound::{Sound, SoundId},
};
use indexmap::IndexMap;

use super::mixer::Mixer;

pub(crate) struct Instances {
	instances: IndexMap<InstanceId, Instance>,
	instances_to_remove: Vec<InstanceId>,
}

impl Instances {
	pub fn new(capacity: usize) -> Self {
		Self {
			instances: IndexMap::with_capacity(capacity),
			instances_to_remove: Vec::with_capacity(capacity),
		}
	}

	pub fn stop_instances_of(&mut self, playable: Playable, settings: StopInstanceSettings) {
		for (_, instance) in &mut self.instances {
			if instance.playable() == playable {
				instance.stop(settings);
			}
		}
	}

	pub fn run_command(
		&mut self,
		command: InstanceCommand,
		sounds: &mut IndexMap<SoundId, Sound>,
		arrangements: &mut IndexMap<ArrangementId, Arrangement>,
		groups: &Groups,
	) {
		match command {
			InstanceCommand::Play(instance_id, instance) => match instance.playable() {
				Playable::Sound(id) => {
					if let Some(sound) = sounds.get_mut(&id) {
						if !sound.cooling_down() {
							self.instances.insert(instance_id, instance);
							sound.start_cooldown();
						}
					}
				}
				Playable::Arrangement(id) => {
					if let Some(arrangement) = arrangements.get_mut(&id) {
						if !arrangement.cooling_down() {
							self.instances.insert(instance_id, instance);
							arrangement.start_cooldown();
						}
					}
				}
			},
			InstanceCommand::SetInstanceVolume(id, value) => {
				if let Some(instance) = self.instances.get_mut(&id) {
					instance.set_volume(value);
				}
			}
			InstanceCommand::SetInstancePitch(id, value) => {
				if let Some(instance) = self.instances.get_mut(&id) {
					instance.set_pitch(value);
				}
			}
			InstanceCommand::SetInstancePanning(id, value) => {
				if let Some(instance) = self.instances.get_mut(&id) {
					instance.set_panning(value);
				}
			}
			InstanceCommand::SeekInstance(id, offset) => {
				if let Some(instance) = self.instances.get_mut(&id) {
					instance.seek(offset);
				}
			}
			InstanceCommand::SeekInstanceTo(id, position) => {
				if let Some(instance) = self.instances.get_mut(&id) {
					instance.seek_to(position);
				}
			}
			InstanceCommand::PauseInstance(id, settings) => {
				if let Some(instance) = self.instances.get_mut(&id) {
					instance.pause(settings);
				}
			}
			InstanceCommand::ResumeInstance(id, settings) => {
				if let Some(instance) = self.instances.get_mut(&id) {
					instance.resume(settings);
				}
			}
			InstanceCommand::StopInstance(id, settings) => {
				if let Some(instance) = self.instances.get_mut(&id) {
					instance.stop(settings);
				}
			}
			InstanceCommand::PauseInstancesOf(playable, settings) => {
				for (_, instance) in &mut self.instances {
					if instance.playable() == playable {
						instance.pause(settings);
					}
				}
			}
			InstanceCommand::ResumeInstancesOf(playable, settings) => {
				for (_, instance) in &mut self.instances {
					if instance.playable() == playable {
						instance.resume(settings);
					}
				}
			}
			InstanceCommand::StopInstancesOf(playable, settings) => {
				self.stop_instances_of(playable, settings);
			}
			InstanceCommand::PauseGroup(id, settings) => {
				for (_, instance) in &mut self.instances {
					if instance.is_in_group(id, sounds, arrangements, groups) {
						instance.pause(settings);
					}
				}
			}
			InstanceCommand::ResumeGroup(id, settings) => {
				for (_, instance) in &mut self.instances {
					if instance.is_in_group(id, sounds, arrangements, groups) {
						instance.resume(settings);
					}
				}
			}
			InstanceCommand::StopGroup(id, settings) => {
				for (_, instance) in &mut self.instances {
					if instance.is_in_group(id, sounds, arrangements, groups) {
						instance.stop(settings);
					}
				}
			}
			InstanceCommand::PauseInstancesOfSequence(id, settings) => {
				for (_, instance) in &mut self.instances {
					if instance.sequence_id() == Some(id) {
						instance.pause(settings);
					}
				}
			}
			InstanceCommand::ResumeInstancesOfSequence(id, settings) => {
				for (_, instance) in &mut self.instances {
					if instance.sequence_id() == Some(id) {
						instance.resume(settings);
					}
				}
			}
			InstanceCommand::StopInstancesOfSequence(id, settings) => {
				for (_, instance) in &mut self.instances {
					if instance.sequence_id() == Some(id) {
						instance.stop(settings);
					}
				}
			}
		}
	}

	pub fn process(
		&mut self,
		dt: f64,
		sounds: &IndexMap<SoundId, Sound>,
		arrangements: &IndexMap<ArrangementId, Arrangement>,
		mixer: &mut Mixer,
		parameters: &Parameters,
	) {
		for (instance_id, instance) in &mut self.instances {
			if instance.playing() {
				mixer.add_input(
					instance
						.track_index()
						.get(instance.playable(), sounds, arrangements),
					instance.get_sample(sounds, arrangements),
				);
			}
			if instance.finished() {
				self.instances_to_remove.push(*instance_id);
			}
			instance.update(dt, parameters, sounds, arrangements);
		}
		for instance_id in self.instances_to_remove.drain(..) {
			self.instances.remove(&instance_id);
		}
	}
}
