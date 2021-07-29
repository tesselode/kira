use atomic_arena::{Arena, Controller};
use ringbuf::Producer;

use crate::{
	frame::Frame,
	manager::command::InstanceCommand,
	sound::instance::{Instance, InstanceState},
};

use super::{parameters::Parameters, sounds::Sounds};

pub(crate) struct Instances {
	instances: Arena<Instance>,
	unused_instance_producer: Producer<Instance>,
}

impl Instances {
	pub fn new(capacity: usize, unused_instance_producer: Producer<Instance>) -> Self {
		Self {
			instances: Arena::new(capacity),
			unused_instance_producer,
		}
	}

	pub fn controller(&self) -> Controller {
		self.instances.controller()
	}

	pub fn on_start_processing(&mut self) {
		if self.unused_instance_producer.is_full() {
			return;
		}
		for (_, instance) in self
			.instances
			.drain_filter(|instance| instance.state() == InstanceState::Stopped)
		{
			if self.unused_instance_producer.push(instance).is_err() {
				panic!("Unused instance producer is full")
			}
			if self.unused_instance_producer.is_full() {
				return;
			}
		}
	}

	pub fn run_command(&mut self, command: InstanceCommand) {
		match command {
			InstanceCommand::Add(id, instance) => self
				.instances
				.insert_with_index(id.0, instance)
				.expect("Instance arena is full"),
			InstanceCommand::SetVolume(id, volume) => {
				if let Some(instance) = self.instances.get_mut(id.0) {
					instance.set_volume(volume);
				}
			}
			InstanceCommand::SetPlaybackRate(id, playback_rate) => {
				if let Some(instance) = self.instances.get_mut(id.0) {
					instance.set_playback_rate(playback_rate);
				}
			}
			InstanceCommand::SetPanning(id, panning) => {
				if let Some(instance) = self.instances.get_mut(id.0) {
					instance.set_panning(panning);
				}
			}
		}
	}

	pub fn process(
		&mut self,
		sample_count: u64,
		dt: f64,
		sounds: &Sounds,
		parameters: &Parameters,
	) -> Frame {
		self.instances
			.iter_mut()
			.fold(Frame::from_mono(0.0), |previous, (_, instance)| {
				previous + instance.process(sample_count, dt, sounds, parameters)
			})
	}
}
