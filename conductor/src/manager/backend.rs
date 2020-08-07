use super::{AudioManagerSettings, InstanceSettings};
use crate::{
	id::{InstanceId, SoundId},
	project::Project,
	stereo_sample::StereoSample,
};
use indexmap::IndexMap;
use ringbuf::Consumer;

pub enum Command {
	PlaySound(SoundId, InstanceId, InstanceSettings),
}

struct Instance {
	sound_id: SoundId,
	volume: f32,
	pitch: f32,
	position: f32,
}

impl Instance {
	fn new(sound_id: SoundId, settings: InstanceSettings) -> Self {
		Self {
			sound_id,
			volume: settings.volume,
			pitch: settings.pitch,
			position: 0.0,
		}
	}
}

pub struct Backend {
	dt: f32,
	project: Project,
	instances: IndexMap<InstanceId, Instance>,
	command_consumer: Consumer<Command>,
	instances_to_remove: Vec<InstanceId>,
}

impl Backend {
	pub fn new(
		sample_rate: u32,
		project: Project,
		command_consumer: Consumer<Command>,
		settings: AudioManagerSettings,
	) -> Self {
		Self {
			dt: 1.0 / sample_rate as f32,
			project,
			instances: IndexMap::with_capacity(settings.num_instances),
			command_consumer,
			instances_to_remove: Vec::with_capacity(settings.num_instances),
		}
	}

	pub fn process_commands(&mut self) {
		while let Some(command) = self.command_consumer.pop() {
			match command {
				Command::PlaySound(sound_id, instance_id, settings) => {
					self.instances
						.insert(instance_id, Instance::new(sound_id, settings));
				}
			}
		}
	}

	pub fn process(&mut self) -> StereoSample {
		self.process_commands();
		let mut out = StereoSample::from_mono(0.0);
		for (instance_id, instance) in &mut self.instances {
			let sound = self.project.get_sound(&instance.sound_id);
			out += sound.get_sample_at_position(instance.position) * instance.volume;
			instance.position += instance.pitch * self.dt;
			if instance.position >= sound.duration() {
				self.instances_to_remove.push(*instance_id);
			}
		}
		for instance_id in self.instances_to_remove.drain(..) {
			self.instances.remove(&instance_id);
		}
		out
	}
}
