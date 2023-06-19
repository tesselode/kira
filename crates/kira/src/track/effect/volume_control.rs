//! Adjusts the volume of audio.

mod builder;
mod handle;

pub use builder::*;
pub use handle::*;

use ringbuf::HeapConsumer;

use crate::{
	clock::clock_info::ClockInfoProvider,
	dsp::Frame,
	modulator::value_provider::ModulatorValueProvider,
	tween::{Parameter, Tween, Value},
	Amplitude, Decibels,
};

use super::Effect;

enum Command {
	SetVolume(Value<Decibels>, Tween),
}

struct VolumeControl {
	command_consumer: HeapConsumer<Command>,
	volume: Parameter<Decibels>,
}

impl VolumeControl {
	fn new(builder: VolumeControlBuilder, command_consumer: HeapConsumer<Command>) -> Self {
		Self {
			command_consumer,
			volume: Parameter::new(builder.0, Decibels(0.0)),
		}
	}
}

impl Effect for VolumeControl {
	fn on_start_processing(&mut self) {
		while let Some(command) = self.command_consumer.pop() {
			match command {
				Command::SetVolume(volume, tween) => self.volume.set(volume, tween),
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
		self.volume
			.update(dt, clock_info_provider, modulator_value_provider);
		input * Amplitude::from(self.volume.value()).0 as f32
	}
}
