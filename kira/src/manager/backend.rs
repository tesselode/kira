use ringbuf::Consumer;

use crate::{
	sequence::instance::SequenceInstance,
	sound::instance::{Instance, InstancePlaybackState},
	Frame,
};

use super::{command::Command, AudioManagerSettings};

pub struct Backend {
	sample_rate: u32,
	dt: f64,
	command_consumer: Consumer<Command>,
	instances: Vec<Instance>,
	sequence_instances: Vec<SequenceInstance>,
	sequence_instance_command_queue: Vec<Command>,
}

impl Backend {
	pub(crate) fn new(
		sample_rate: u32,
		command_consumer: Consumer<Command>,
		settings: AudioManagerSettings,
	) -> Self {
		Self {
			sample_rate,
			dt: 1.0 / sample_rate as f64,
			command_consumer,
			instances: Vec::with_capacity(settings.num_instances),
			sequence_instances: Vec::with_capacity(settings.num_sequences),
			sequence_instance_command_queue: Vec::with_capacity(settings.num_commands),
		}
	}

	fn run_command(
		instances: &mut Vec<Instance>,
		sequence_instances: &mut Vec<SequenceInstance>,
		command: Command,
	) {
		match command {
			Command::StartInstance { instance } => {
				if instances.len() < instances.capacity() {
					instances.push(instance);
				}
			}
			Command::StartSequenceInstance(mut instance) => {
				if sequence_instances.len() < sequence_instances.capacity() {
					instance.start();
					sequence_instances.push(instance);
				}
			}
		}
	}

	fn update_sequence_instances(&mut self) {
		for sequence_instance in &mut self.sequence_instances {
			sequence_instance.update(self.dt, &mut self.sequence_instance_command_queue);
		}
		for command in self.sequence_instance_command_queue.drain(..) {
			Self::run_command(&mut self.instances, &mut self.sequence_instances, command);
		}
		self.sequence_instances
			.retain(|instance| !instance.finished());
	}

	pub fn process(&mut self) -> Frame {
		while let Some(command) = self.command_consumer.pop() {
			Self::run_command(&mut self.instances, &mut self.sequence_instances, command);
		}

		self.update_sequence_instances();

		let dt = self.dt;
		let output = self
			.instances
			.iter_mut()
			.fold(Frame::from_mono(0.0), |previous, instance| {
				previous + instance.process(dt)
			});
		self.instances
			.retain(|instance| instance.state() != InstancePlaybackState::Stopped);
		output
	}
}
