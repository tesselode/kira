use std::marker::PhantomData;

use atomig::{Atomic, Ordering};
use basedrop::{Handle, Shared};

pub(crate) struct ParameterState {
	value: Atomic<f64>,
}

#[derive(Clone)]
pub struct Parameter<T: From<f64> + Into<f64>> {
	state: Shared<ParameterState>,
	_phantom: PhantomData<T>,
}

impl<T: From<f64> + Into<f64>> Parameter<T> {
	pub(crate) fn new(value: T, collector_handle: &Handle) -> Self {
		Self {
			state: Shared::new(
				collector_handle,
				ParameterState {
					value: Atomic::new(value.into()),
				},
			),
			_phantom: PhantomData,
		}
	}

	pub fn get(&self) -> T {
		self.state.value.load(Ordering::Relaxed).into()
	}

	pub fn set(&self, value: T) {
		self.state.value.store(value.into(), Ordering::Relaxed);
	}
}
