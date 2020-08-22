use crate::tween::Tween;

struct TweenState {
	tween: Tween,
	start: f64,
	target: f64,
	progress: f64,
}

pub struct Parameter {
	value: f64,
	tween_state: Option<TweenState>,
}

impl Parameter {
	pub fn new(value: f64) -> Self {
		Self {
			value,
			tween_state: None,
		}
	}

	pub fn value(&self) -> f64 {
		self.value
	}

	pub fn set(&mut self, target: f64, tween: Option<Tween>) {
		if let Some(tween) = tween {
			self.tween_state = Some(TweenState {
				tween,
				start: self.value,
				target: target,
				progress: 0.0,
			});
		} else {
			self.value = target;
		}
	}

	pub fn update(&mut self, dt: f64) -> bool {
		if let Some(tween_state) = &mut self.tween_state {
			let duration = tween_state.tween.0;
			tween_state.progress += dt / duration;
			tween_state.progress = tween_state.progress.min(1.0);
			self.value =
				tween_state.start + (tween_state.target - tween_state.start) * tween_state.progress;
			if tween_state.progress >= 1.0 {
				self.tween_state = None;
				return true;
			}
		}
		false
	}
}
