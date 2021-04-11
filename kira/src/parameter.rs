pub mod tween;

use std::sync::atomic::AtomicBool;

use atomig::{Atomic, Ordering};
use basedrop::{Handle, Shared};

use crate::util::lerp;

use self::tween::{Easing, Tween};

pub(crate) struct ParameterState {
	value: Atomic<f64>,
	tweening: AtomicBool,
	tween_start: Atomic<f64>,
	tween_end: Atomic<f64>,
	tween_duration: Atomic<f64>,
	tween_progress: Atomic<f64>,
	tween_easing: Atomic<Easing>,
}

#[derive(Clone)]
pub struct Parameter {
	state: Shared<ParameterState>,
}

impl Parameter {
	pub(crate) fn new(value: f64, collector_handle: &Handle) -> Self {
		Self {
			state: Shared::new(
				collector_handle,
				ParameterState {
					value: Atomic::new(value.into()),
					tweening: AtomicBool::new(false),
					tween_start: Atomic::new(0.0),
					tween_end: Atomic::new(0.0),
					tween_duration: Atomic::new(0.0),
					tween_progress: Atomic::new(0.0),
					tween_easing: Atomic::new(Easing::Linear),
				},
			),
		}
	}

	pub fn get(&self) -> f64 {
		self.state.value.load(Ordering::Relaxed).into()
	}

	pub fn set(&self, value: f64) {
		self.state.value.store(value.into(), Ordering::Relaxed);
	}

	pub fn tween(&self, target: f64, tween: Tween) {
		self.state.tweening.store(true, Ordering::Relaxed);
		self.state
			.tween_start
			.store(self.state.value.load(Ordering::Relaxed), Ordering::Relaxed);
		self.state.tween_end.store(target.into(), Ordering::Relaxed);
		self.state
			.tween_duration
			.store(tween.duration, Ordering::Relaxed);
		self.state.tween_progress.store(0.0, Ordering::Relaxed);
		self.state
			.tween_easing
			.store(tween.easing, Ordering::Relaxed);
	}

	pub fn update(&mut self, dt: f64) {
		if self.state.tweening.load(Ordering::Relaxed) {
			let duration = self.state.tween_duration.load(Ordering::Relaxed);
			let start = self.state.tween_start.load(Ordering::Relaxed);
			let end = self.state.tween_end.load(Ordering::Relaxed);
			let easing = self.state.tween_easing.load(Ordering::Relaxed);
			let new_progress = self.state.tween_progress.load(Ordering::Relaxed) + dt;
			self.state
				.tween_progress
				.store(new_progress, Ordering::Relaxed);
			if new_progress >= duration {
				self.state.tweening.store(false, Ordering::Relaxed);
				self.state.value.store(
					self.state.tween_end.load(Ordering::Relaxed),
					Ordering::Relaxed,
				);
			} else {
				self.state.value.store(
					lerp(start, end, easing.ease(new_progress / duration)),
					Ordering::Relaxed,
				);
			}
		}
	}
}
