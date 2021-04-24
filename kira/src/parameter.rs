pub mod tween;

use std::sync::{atomic::AtomicBool, Arc};

use atomig::{Atomic, Ordering};

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
	state: Arc<ParameterState>,
}

impl Parameter {
	pub(crate) fn new(value: f64) -> Self {
		Self {
			state: Arc::new(ParameterState {
				value: Atomic::new(value.into()),
				tweening: AtomicBool::new(false),
				tween_start: Atomic::new(0.0),
				tween_end: Atomic::new(0.0),
				tween_duration: Atomic::new(0.0),
				tween_progress: Atomic::new(0.0),
				tween_easing: Atomic::new(Easing::Linear),
			}),
		}
	}

	pub fn get(&self) -> f64 {
		self.state.value.load(Ordering::SeqCst).into()
	}

	pub fn set(&self, value: f64) {
		self.state.value.store(value.into(), Ordering::SeqCst);
	}

	pub fn tween(&self, target: f64, tween: Tween) {
		self.state.tweening.store(true, Ordering::SeqCst);
		self.state
			.tween_start
			.store(self.state.value.load(Ordering::SeqCst), Ordering::SeqCst);
		self.state.tween_end.store(target.into(), Ordering::SeqCst);
		self.state
			.tween_duration
			.store(tween.duration, Ordering::SeqCst);
		self.state.tween_progress.store(0.0, Ordering::SeqCst);
		self.state
			.tween_easing
			.store(tween.easing, Ordering::SeqCst);
	}

	pub fn update(&mut self, dt: f64) {
		if self.state.tweening.load(Ordering::SeqCst) {
			let duration = self.state.tween_duration.load(Ordering::SeqCst);
			let start = self.state.tween_start.load(Ordering::SeqCst);
			let end = self.state.tween_end.load(Ordering::SeqCst);
			let easing = self.state.tween_easing.load(Ordering::SeqCst);
			let new_progress = self.state.tween_progress.load(Ordering::SeqCst) + dt;
			self.state
				.tween_progress
				.store(new_progress, Ordering::SeqCst);
			if new_progress >= duration {
				self.state.tweening.store(false, Ordering::SeqCst);
				self.state.value.store(
					self.state.tween_end.load(Ordering::SeqCst),
					Ordering::SeqCst,
				);
			} else {
				self.state.value.store(
					lerp(start, end, easing.ease(new_progress / duration)),
					Ordering::SeqCst,
				);
			}
		}
	}
}

#[cfg(feature = "log_drops")]
impl Drop for Parameter {
	fn drop(&mut self) {
		println!("dropped parameter");
	}
}
