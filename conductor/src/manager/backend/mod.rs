mod metronome;

use super::{AudioManagerSettings, Event, InstanceId};
use crate::{
	project::{Project, SoundId},
	stereo_sample::StereoSample,
};
use metronome::Metronome;
use ringbuf::{Consumer, Producer};
use std::collections::HashMap;

struct Instance {
	sound_id: SoundId,
	position: f32,
}

impl Instance {
	pub fn new(sound_id: SoundId) -> Self {
		Self {
			sound_id,
			position: 0.0,
		}
	}
}

pub enum Command {
	PlaySound(SoundId, InstanceId),
	StartMetronome,
}

pub struct Backend {
	dt: f32,
	project: Project,
	instances: HashMap<InstanceId, Instance>,
	command_consumer: Consumer<Command>,
	event_producer: Producer<Event>,
	metronome: Metronome,
	metronome_event_intervals: Vec<f32>,
}

impl Backend {
	pub fn new(
		sample_rate: u32,
		project: Project,
		settings: AudioManagerSettings,
		command_consumer: Consumer<Command>,
		event_producer: Producer<Event>,
	) -> Self {
		Self {
			dt: 1.0 / sample_rate as f32,
			project,
			instances: HashMap::with_capacity(settings.num_instances),
			command_consumer,
			event_producer,
			metronome: Metronome::new(settings.tempo),
			metronome_event_intervals: settings.metronome_event_intervals,
		}
	}

	fn play_sound(&mut self, sound_id: SoundId, instance_id: InstanceId) {
		if self.instances.len() < self.instances.capacity() {
			self.instances.insert(instance_id, Instance::new(sound_id));
		}
	}

	pub fn process_commands(&mut self) {
		while let Some(command) = self.command_consumer.pop() {
			match command {
				Command::PlaySound(sound_id, instance_id) => self.play_sound(sound_id, instance_id),
				Command::StartMetronome => self.metronome.start(),
			}
		}
	}

	pub fn update_metronome(&mut self) {
		self.metronome.update(self.dt);
		for interval in &self.metronome_event_intervals {
			if self.metronome.interval_passed(*interval) {
				match self
					.event_producer
					.push(Event::MetronomeInterval(*interval))
				{
					Ok(_) => {}
					Err(_) => {}
				}
			}
		}
	}

	pub fn process(&mut self) -> StereoSample {
		self.process_commands();
		self.update_metronome();
		let mut out = StereoSample::from_mono(0.0);
		let mut instance_ids_to_remove = vec![];
		for (instance_id, instance) in &mut self.instances {
			let sound = self.project.get_sound(instance.sound_id);
			out += sound.get_sample_at_position(instance.position);
			instance.position += self.dt;
			if instance.position >= sound.duration() {
				instance_ids_to_remove.push(*instance_id);
			}
		}
		for instance_id in instance_ids_to_remove {
			self.instances.remove(&instance_id);
		}
		out
	}
}
