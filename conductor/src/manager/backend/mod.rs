mod metronome;

use super::{AudioManagerSettings, Event, InstanceId, InstanceSettings};
use crate::{
	project::{Project, SoundId},
	stereo_sample::StereoSample,
};
use bimap::BiMap;
use generational_arena::{Arena, Index};
use metronome::Metronome;
use ringbuf::{Consumer, Producer};

struct Instance {
	sound_id: SoundId,
	position: f32,
	volume: f32,
	pitch: f32,
}

impl Instance {
	pub fn new(sound_id: SoundId, settings: InstanceSettings) -> Self {
		Self {
			sound_id,
			position: 0.0,
			volume: settings.volume,
			pitch: settings.pitch,
		}
	}
}

pub enum Command {
	PlaySound(SoundId, InstanceId, InstanceSettings),
	SetInstanceVolume(InstanceId, f32),
	SetInstancePitch(InstanceId, f32),
	StartMetronome,
}

pub struct Backend {
	dt: f32,
	project: Project,
	instances: Arena<Instance>,
	instance_ids: BiMap<InstanceId, Index>,
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
			instances: Arena::with_capacity(settings.num_instances),
			instance_ids: BiMap::with_capacity(settings.num_instances),
			command_consumer,
			event_producer,
			metronome: Metronome::new(settings.tempo),
			metronome_event_intervals: settings.metronome_event_intervals,
		}
	}

	fn play_sound(
		&mut self,
		sound_id: SoundId,
		instance_id: Option<InstanceId>,
		settings: InstanceSettings,
	) {
		if self.instances.len() < self.instances.capacity() {
			let index = self.instances.insert(Instance::new(sound_id, settings));
			if let Some(id) = instance_id {
				self.instance_ids.insert(id, index);
			}
		}
	}

	fn get_instance(&mut self, instance_id: InstanceId) -> Option<&mut Instance> {
		if let Some(index) = self.instance_ids.get_by_left(&instance_id) {
			return self.instances.get_mut(*index);
		}
		None
	}

	fn remove_instance(&mut self, index: Index) {
		self.instances.remove(index);
		self.instance_ids.remove_by_right(&index);
	}

	fn set_instance_volume(&mut self, instance_id: InstanceId, volume: f32) {
		if let Some(instance) = self.get_instance(instance_id) {
			instance.volume = volume;
		}
	}

	fn set_instance_pitch(&mut self, instance_id: InstanceId, pitch: f32) {
		if let Some(instance) = self.get_instance(instance_id) {
			instance.pitch = pitch;
		}
	}

	pub fn process_commands(&mut self) {
		while let Some(command) = self.command_consumer.pop() {
			match command {
				Command::PlaySound(sound_id, instance_id, settings) => {
					self.play_sound(sound_id, Some(instance_id), settings)
				}
				Command::SetInstanceVolume(instance_id, volume) => {
					self.set_instance_volume(instance_id, volume)
				}
				Command::SetInstancePitch(instance_id, pitch) => {
					self.set_instance_pitch(instance_id, pitch)
				}
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
		let mut instance_indices_to_remove = vec![];
		for (index, instance) in &mut self.instances {
			let sound = self.project.get_sound(instance.sound_id);
			out += sound.get_sample_at_position(instance.position) * instance.volume;
			instance.position += instance.pitch * self.dt;
			if instance.position >= sound.duration() {
				instance_indices_to_remove.push(index);
			}
		}
		for index in instance_indices_to_remove {
			self.remove_instance(index);
		}
		out
	}
}
