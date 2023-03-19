mod builder;
mod handle;

pub use builder::*;
pub use handle::*;

use crate::clock::clock_info::ClockInfoProvider;

use super::Modulator;

struct Tweener {
	state: State,
}

impl Tweener {
	fn new(initial_value: f64) -> Self {
		Self {
			state: State::Idle {
				value: initial_value,
			},
		}
	}
}

impl Modulator for Tweener {
	fn process(&mut self, dt: f64, clock_info_provider: &ClockInfoProvider) -> f64 {
		match &mut self.state {
			State::Idle { value } => *value,
		}
	}

	fn finished(&self) -> bool {
		false
	}
}

enum State {
	Idle { value: f64 },
}
