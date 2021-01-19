use crate::{
	command::InstanceCommand,
	group::groups::Groups,
	instance::{Instance, InstanceId, StopInstanceSettings},
	parameter::Parameters,
	playable::{PlayableId, Playables},
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

	pub fn stop_instances_of(&mut self, playable: PlayableId, settings: StopInstanceSettings) {
		for (_, instance) in &mut self.instances {
			if instance.playable_id() == playable {
				instance.stop(settings);
			}
		}
	}

	pub fn run_command(
		&mut self,
		command: InstanceCommand,
		playables: &mut Playables,
		all_groups: &Groups,
	) {
		match command {
			InstanceCommand::Play(instance_id, instance) => {
				if let Some(mut playable) = playables.playable_mut(instance.playable_id()) {
					if !playable.cooling_down() {
						// if we're at the instance limit, remove the instance that was
						// started the longest time ago.
						if self.instances.len() >= self.instances.capacity() {
							self.instances.shift_remove_index(0);
						}
						self.instances.insert(instance_id, instance);
						playable.start_cooldown();
					}
				}
			}
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
					if instance.playable_id() == playable {
						instance.pause(settings);
					}
				}
			}
			InstanceCommand::ResumeInstancesOf(playable, settings) => {
				for (_, instance) in &mut self.instances {
					if instance.playable_id() == playable {
						instance.resume(settings);
					}
				}
			}
			InstanceCommand::StopInstancesOf(playable, settings) => {
				self.stop_instances_of(playable, settings);
			}
			InstanceCommand::PauseGroup(id, settings) => {
				for (_, instance) in &mut self.instances {
					if let Some(playable) = playables.playable(instance.playable_id()) {
						if playable.is_in_group(id, all_groups) {
							instance.pause(settings);
						}
					}
				}
			}
			InstanceCommand::ResumeGroup(id, settings) => {
				for (_, instance) in &mut self.instances {
					if let Some(playable) = playables.playable(instance.playable_id()) {
						if playable.is_in_group(id, all_groups) {
							instance.resume(settings);
						}
					}
				}
			}
			InstanceCommand::StopGroup(id, settings) => {
				for (_, instance) in &mut self.instances {
					if let Some(playable) = playables.playable(instance.playable_id()) {
						if playable.is_in_group(id, all_groups) {
							instance.stop(settings);
						}
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
		playables: &Playables,
		mixer: &mut Mixer,
		parameters: &Parameters,
	) {
		for (instance_id, instance) in &mut self.instances {
			if instance.playing() {
				mixer.add_input(instance.track_index(), instance.get_sample(playables));
			}
			if instance.finished() {
				self.instances_to_remove.push(*instance_id);
			}
			instance.update(dt, parameters);
		}
		for instance_id in self.instances_to_remove.drain(..) {
			self.instances.shift_remove(&instance_id);
		}
	}
}
