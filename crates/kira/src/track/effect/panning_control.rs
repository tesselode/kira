//! Adjusts the panning of audio.

mod builder;
mod handle;

pub use builder::*;
pub use handle::*;

use ringbuf::HeapConsumer;

use crate::{
	clock::clock_info::ClockInfoProvider,
	dsp::Frame,
	modulator::value_provider::ModulatorValueProvider,
	parameter::{Parameter, Value},
	tween::Tween,
};

use super::Effect;

enum Command {
	SetPanning(Value<f64>, Tween),
}

struct PanningControl {
	command_consumer: HeapConsumer<Command>,
	panning: Parameter,
}

impl PanningControl {
	fn new(builder: PanningControlBuilder, command_consumer: HeapConsumer<Command>) -> Self {
		Self {
			command_consumer,
			panning: Parameter::new(builder.0, 0.5),
		}
	}
}

impl Effect for PanningControl {
	fn on_start_processing(&mut self) {
		while let Some(command) = self.command_consumer.pop() {
			match command {
				Command::SetPanning(panning, tween) => self.panning.set(panning, tween),
			}
		}
	}

	fn process(
		&mut self,
		input: Frame,
		dt: f64,
		clock_info_provider: &ClockInfoProvider,
		modulator_value_provider: &ModulatorValueProvider,
	) -> Frame {
		self.panning
			.update(dt, clock_info_provider, modulator_value_provider);
		input.panned(self.panning.value() as f32)
	}
}
