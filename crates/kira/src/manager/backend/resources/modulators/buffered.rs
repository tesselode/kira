use crate::{
	clock::clock_info::ClockInfoProvider,
	manager::backend::Renderer,
	modulator::{value_provider::ModulatorValueProvider, Modulator},
};

pub(crate) struct BufferedModulator {
	modulator: Box<dyn Modulator>,
	values: Vec<f64>,
}

impl BufferedModulator {
	pub(crate) fn new(modulator: Box<dyn Modulator>) -> Self {
		Self {
			modulator,
			values: Vec::with_capacity(Renderer::INTERNAL_BUFFER_SIZE),
		}
	}

	pub(super) fn clear_buffer(&mut self) {
		self.values.clear();
	}

	pub(super) fn on_start_processing(&mut self) {
		self.modulator.on_start_processing();
	}

	pub(super) fn update(
		&mut self,
		dt: f64,
		clock_info_provider: &ClockInfoProvider,
		modulator_value_provider: &ModulatorValueProvider,
	) {
		self.modulator
			.update(dt, clock_info_provider, modulator_value_provider);
		self.values.push(self.modulator.value());
	}

	pub(super) fn finished(&self) -> bool {
		self.modulator.finished()
	}

	pub(crate) fn latest_value(&self) -> f64 {
		self.modulator.value()
	}

	pub(crate) fn value_at_index(&self, index: usize) -> f64 {
		self.values[index]
	}
}
