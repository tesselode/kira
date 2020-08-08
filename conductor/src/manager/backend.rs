use super::{AudioManagerSettings, Event, InstanceSettings};
use crate::{
	id::{InstanceId, MetronomeId, SoundId},
	project::Project,
	stereo_sample::StereoSample,
};
use indexmap::IndexMap;
use ringbuf::{Consumer, Producer};

pub enum Command {
	PlaySound(SoundId, InstanceId, InstanceSettings),
	StartMetronome(MetronomeId),
	PauseMetronome(MetronomeId),
	StopMetronome(MetronomeId),
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
	event_producer: Producer<Event>,
	metronome_interval_event_collector: Vec<f32>,
	instances_to_remove: Vec<InstanceId>,
}

impl Backend {
	pub fn new(
		sample_rate: u32,
		project: Project,
		command_consumer: Consumer<Command>,
		event_producer: Producer<Event>,
		settings: AudioManagerSettings,
	) -> Self {
		Self {
			dt: 1.0 / sample_rate as f32,
			project,
			instances: IndexMap::with_capacity(settings.num_instances),
			command_consumer,
			event_producer,
			metronome_interval_event_collector: Vec::with_capacity(settings.num_events),
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
				Command::StartMetronome(id) => {
					self.project.metronomes.get_mut(&id).unwrap().start();
				}
				Command::PauseMetronome(id) => {
					self.project.metronomes.get_mut(&id).unwrap().pause();
				}
				Command::StopMetronome(id) => {
					self.project.metronomes.get_mut(&id).unwrap().stop();
				}
			}
		}
	}

	pub fn update_metronomes(&mut self) {
		for (id, metronome) in &mut self.project.metronomes {
			metronome.update(self.dt, &mut self.metronome_interval_event_collector);
			for interval in self.metronome_interval_event_collector.drain(..) {
				match self
					.event_producer
					.push(Event::MetronomeIntervalPassed(*id, interval))
				{
					Ok(_) => {}
					Err(_) => {}
				}
			}
		}
	}

	pub fn process(&mut self) -> StereoSample {
		self.process_commands();
		self.update_metronomes();
		let mut out = StereoSample::from_mono(0.0);
		for (instance_id, instance) in &mut self.instances {
			let sound = self.project.sounds.get(&instance.sound_id).unwrap();
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
