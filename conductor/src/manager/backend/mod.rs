use super::AudioManagerSettings;
use crate::{
	project::{Project, SoundId},
	stereo_sample::StereoSample,
};
use ringbuf::Consumer;

#[derive(Eq, PartialEq)]
enum InstanceState {
	Stopped,
	Playing,
}

struct Instance {
	sound_id: Option<SoundId>,
	position: f32,
	state: InstanceState,
}

impl Instance {
	pub fn new() -> Self {
		Self {
			sound_id: None,
			position: 0.0,
			state: InstanceState::Stopped,
		}
	}
}

pub enum Command {
	PlaySound(SoundId),
}

pub struct Backend {
	dt: f32,
	project: Project,
	instances: Vec<Instance>,
	command_consumer: Consumer<Command>,
}

impl Backend {
	pub fn new(
		sample_rate: u32,
		project: Project,
		settings: AudioManagerSettings,
		command_consumer: Consumer<Command>,
	) -> Self {
		let mut instances = vec![];
		for _ in 0..settings.num_instances {
			instances.push(Instance::new());
		}
		Self {
			dt: 1.0 / sample_rate as f32,
			project,
			instances,
			command_consumer,
		}
	}

	fn pick_instance(&mut self) -> Option<&mut Instance> {
		self.instances
			.iter_mut()
			.find(|instance| instance.state == InstanceState::Stopped)
	}

	fn play_sound(&mut self, sound_id: SoundId) {
		if let Some(instance) = self.pick_instance() {
			instance.sound_id = Some(sound_id);
			instance.position = 0.0;
			instance.state = InstanceState::Playing;
		}
	}

	pub fn process_commands(&mut self) {
		while let Some(command) = self.command_consumer.pop() {
			match command {
				Command::PlaySound(sound_id) => self.play_sound(sound_id),
			}
		}
	}

	pub fn process(&mut self) -> StereoSample {
		self.process_commands();
		let mut out = StereoSample::from_mono(0.0);
		for instance in &mut self.instances {
			if instance.state == InstanceState::Playing {
				let sound = self.project.get_sound(instance.sound_id.unwrap());
				out += sound.get_sample_at_position(instance.position);
				instance.position += self.dt;
				if instance.position >= sound.duration() {
					instance.position = sound.duration();
					instance.state = InstanceState::Stopped;
				}
			}
		}
		out
	}
}
