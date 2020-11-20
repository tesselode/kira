use crate::{
	command::InstanceCommand,
	instance::{Instance, InstanceId},
	parameter::{Parameters, Tween},
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

	pub fn stop_instances_of_sound(&mut self, id: SoundId, fade_tween: Option<Tween>) {
		for (_, instance) in &mut self.instances {
			if instance.sound_id() == id {
				instance.stop(fade_tween);
			}
		}
	}

	pub fn run_command(&mut self, command: InstanceCommand, sounds: &mut IndexMap<SoundId, Sound>) {
		match command {
			InstanceCommand::PlaySound(instance_id, sound_id, sequence_id, settings) => {
				if let Some(sound) = sounds.get_mut(&sound_id) {
					if !sound.cooling_down() {
						self.instances
							.insert(instance_id, Instance::new(sound_id, sequence_id, settings));
						sound.start_cooldown();
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
			InstanceCommand::PauseInstance(id, fade_tween) => {
				if let Some(instance) = self.instances.get_mut(&id) {
					instance.pause(fade_tween);
				}
			}
			InstanceCommand::ResumeInstance(id, fade_tween) => {
				if let Some(instance) = self.instances.get_mut(&id) {
					instance.resume(fade_tween);
				}
			}
			InstanceCommand::StopInstance(id, fade_tween) => {
				if let Some(instance) = self.instances.get_mut(&id) {
					instance.stop(fade_tween);
				}
			}
			InstanceCommand::PauseInstancesOfSound(id, fade_tween) => {
				for (_, instance) in &mut self.instances {
					if instance.sound_id() == id {
						instance.pause(fade_tween);
					}
				}
			}
			InstanceCommand::ResumeInstancesOfSound(id, fade_tween) => {
				for (_, instance) in &mut self.instances {
					if instance.sound_id() == id {
						instance.resume(fade_tween);
					}
				}
			}
			InstanceCommand::StopInstancesOfSound(id, fade_tween) => {
				self.stop_instances_of_sound(id, fade_tween);
			}
			InstanceCommand::PauseInstancesOfSequence(id, fade_tween) => {
				for (_, instance) in &mut self.instances {
					if instance.sequence_id() == Some(id) {
						instance.pause(fade_tween);
					}
				}
			}
			InstanceCommand::ResumeInstancesOfSequence(id, fade_tween) => {
				for (_, instance) in &mut self.instances {
					if instance.sequence_id() == Some(id) {
						instance.resume(fade_tween);
					}
				}
			}
			InstanceCommand::StopInstancesOfSequence(id, fade_tween) => {
				for (_, instance) in &mut self.instances {
					if instance.sequence_id() == Some(id) {
						instance.stop(fade_tween);
					}
				}
			}
		}
	}

	pub fn process(
		&mut self,
		dt: f64,
		sounds: &IndexMap<SoundId, Sound>,
		mixer: &mut Mixer,
		parameters: &Parameters,
	) {
		for (instance_id, instance) in &mut self.instances {
			if instance.playing() {
				if let Some(sound) = sounds.get(&instance.sound_id()) {
					mixer.add_input(instance.track_index(), instance.get_sample(sound));
				}
			}
			if instance.finished() {
				self.instances_to_remove.push(*instance_id);
			}
			instance.update(dt, parameters);
		}
		for instance_id in self.instances_to_remove.drain(..) {
			self.instances.remove(&instance_id);
		}
	}
}
