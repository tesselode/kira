mod instance;
mod metronome;

use super::{AudioManagerSettings, Event, InstanceHandle, InstanceSettings};
use crate::{
	project::{Project, SoundId},
	stereo_sample::StereoSample,
};
use bimap::BiMap;
use generational_arena::{Arena, Index};
use instance::Instance;
use metronome::Metronome;
use ringbuf::{Consumer, Producer};

pub enum Command {
	PlaySound(SoundId, InstanceHandle, InstanceSettings),
	SetInstanceVolume(InstanceHandle, f32),
	SetInstancePitch(InstanceHandle, f32),
	StartMetronome,
}

pub struct Backend {
	dt: f32,
	project: Project,
	instances: Arena<Instance>,
	instance_handles: BiMap<InstanceHandle, Index>,
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
			instance_handles: BiMap::with_capacity(settings.num_instances),
			command_consumer,
			event_producer,
			metronome: Metronome::new(settings.tempo),
			metronome_event_intervals: settings.metronome_event_intervals,
		}
	}

	fn play_sound(
		&mut self,
		sound_id: SoundId,
		instance_handle: Option<InstanceHandle>,
		settings: InstanceSettings,
	) {
		if self.instances.len() < self.instances.capacity() {
			let index = self.instances.insert(Instance::new(sound_id, settings));
			if let Some(handle) = instance_handle {
				self.instance_handles.insert(handle, index);
			}
		}
	}

	fn get_instance(&mut self, instance_handle: InstanceHandle) -> Option<&mut Instance> {
		if let Some(index) = self.instance_handles.get_by_left(&instance_handle) {
			return self.instances.get_mut(*index);
		}
		None
	}

	fn remove_instance(&mut self, index: Index) {
		self.instances.remove(index);
		self.instance_handles.remove_by_right(&index);
	}

	fn set_instance_volume(&mut self, instance_handle: InstanceHandle, volume: f32) {
		if let Some(instance) = self.get_instance(instance_handle) {
			instance.volume = volume;
		}
	}

	fn set_instance_pitch(&mut self, instance_handle: InstanceHandle, pitch: f32) {
		if let Some(instance) = self.get_instance(instance_handle) {
			instance.pitch = pitch;
		}
	}

	pub fn process_commands(&mut self) {
		while let Some(command) = self.command_consumer.pop() {
			match command {
				Command::PlaySound(sound_id, instance_handle, settings) => {
					self.play_sound(sound_id, Some(instance_handle), settings)
				}
				Command::SetInstanceVolume(instance_handle, volume) => {
					self.set_instance_volume(instance_handle, volume)
				}
				Command::SetInstancePitch(instance_handle, pitch) => {
					self.set_instance_pitch(instance_handle, pitch)
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
