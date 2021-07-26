use atomic_arena::Arena;
use ringbuf::Producer;

use crate::{
	frame::Frame,
	manager::command::InstanceCommand,
	sound::instance::{Instance, InstanceState},
};

use super::sounds::Sounds;

pub(crate) struct Instances {
	pub instances: Arena<Instance>,
	pub unused_instance_producer: Producer<Instance>,
}

impl Instances {
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
		}
	}

	pub fn process(&mut self, dt: f64, sounds: &Sounds) -> Frame {
		self.instances
			.iter_mut()
			.fold(Frame::from_mono(0.0), |previous, (_, instance)| {
				let sound = &sounds.sounds[instance.sound_id().0];
				previous + instance.process(dt, &sound.data)
			})
	}
}
