pub mod handle;
pub mod tween;

use std::{
	ops::RangeInclusive,
	sync::{
		atomic::{AtomicBool, AtomicU64, Ordering},
		Arc,
	},
	time::Duration,
};

use atomic_arena::Index;

use crate::manager::backend::context::Context;

use self::tween::Tween;

type JustFinishedTween = bool;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ParameterId(pub(crate) Index);

pub(crate) struct ParameterShared {
	value: AtomicU64,
	removed: AtomicBool,
}

impl ParameterShared {
	pub fn new(value: f64) -> Self {
		Self {
			value: AtomicU64::new(value.to_bits()),
			removed: AtomicBool::new(false),
		}
	}

	pub fn value(&self) -> f64 {
		f64::from_bits(self.value.load(Ordering::SeqCst))
	}

	pub fn is_marked_for_removal(&self) -> bool {
		self.removed.load(Ordering::SeqCst)
	}

	pub fn mark_for_removal(&self) {
		self.removed.store(true, Ordering::SeqCst);
	}
}

enum ParameterState {
	Idle,
	Tweening {
		values: RangeInclusive<f64>,
		time: f64,
		tween: Tween,
	},
}

pub(crate) struct Parameter {
	state: ParameterState,
	value: f64,
	shared: Arc<ParameterShared>,
}

impl Parameter {
	pub fn new(value: f64) -> Self {
		Self {
			state: ParameterState::Idle,
			value,
			shared: Arc::new(ParameterShared::new(value)),
		}
	}

	pub fn shared(&self) -> Arc<ParameterShared> {
		self.shared.clone()
	}

	pub fn value(&self) -> f64 {
		self.value
	}

	pub fn set(&mut self, value: f64) {
		self.value = value;
	}

	pub fn tween(
		&mut self,
		context: &Arc<Context>,
		target: f64,
		mut tween: Tween,
		command_sent_time: u64,
	) {
		let time_since_command_sent_samples = context.sample_count() - command_sent_time;
		tween.delay = tween.delay.saturating_sub(Duration::from_secs_f64(
			time_since_command_sent_samples as f64 / context.sample_rate() as f64,
		));
		self.state = ParameterState::Tweening {
			values: self.value..=target,
			time: 0.0,
			tween,
		};
	}

	pub fn on_start_processing(&self) {
		self.shared
			.value
			.store(self.value.to_bits(), Ordering::SeqCst);
	}

	pub fn update(&mut self, dt: f64) -> JustFinishedTween {
		if let ParameterState::Tweening {
			values,
			time,
			tween,
		} = &mut self.state
		{
			*time += dt;
			if *time >= tween.duration.as_secs_f64() + tween.delay.as_secs_f64() {
				self.value = *values.end();
				self.state = ParameterState::Idle;
				return true;
			} else {
				self.value = values.start() + (values.end() - values.start()) * tween.value(*time);
			}
		}
		false
	}
}
