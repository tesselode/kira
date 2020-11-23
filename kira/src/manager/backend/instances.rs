use crate::{
	arrangement::{Arrangement, ArrangementId},
	command::InstanceCommand,
	instance::{Instance, InstanceId},
	parameter::{Parameters, Tween},
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

	pub fn stop_instances_of(&mut self, playable: Playable, fade_tween: Option<Tween>) {
		for (_, instance) in &mut self.instances {
			if instance.playable() == playable {
				instance.stop(fade_tween);
			}
		}
	}

	pub fn run_command(&mut self, command: InstanceCommand, sounds: &mut IndexMap<SoundId, Sound>) {
		match command {
			InstanceCommand::Play(instance_id, playable, sequence_id, settings) => {
				self.instances
					.insert(instance_id, Instance::new(playable, sequence_id, settings));
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
			InstanceCommand::PauseInstancesOf(playable, fade_tween) => {
				for (_, instance) in &mut self.instances {
					if instance.playable() == playable {
						instance.pause(fade_tween);
					}
				}
			}
			InstanceCommand::ResumeInstancesOf(playable, fade_tween) => {
				for (_, instance) in &mut self.instances {
					if instance.playable() == playable {
						instance.resume(fade_tween);
					}
				}
			}
			InstanceCommand::StopInstancesOf(playable, fade_tween) => {
				self.stop_instances_of(playable, fade_tween);
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
		arrangements: &IndexMap<ArrangementId, Arrangement>,
		mixer: &mut Mixer,
		parameters: &Parameters,
	) {
		for (instance_id, instance) in &mut self.instances {
			if instance.playing() {
				mixer.add_input(
					instance.track_index(),
					instance.get_sample(sounds, arrangements),
				);
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
