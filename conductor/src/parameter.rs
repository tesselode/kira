use crate::tween::Tween;

struct TweenState {
	tween: Tween<f32>,
	start: f32,
	target: f32,
	progress: f32,
}

pub struct Parameter {
	value: f32,
	tween_state: Option<TweenState>,
}

impl Parameter {
	pub fn new(value: f32) -> Self {
		Self {
			value,
			tween_state: None,
		}
	}

	pub fn value(&self) -> f32 {
		self.value
	}

	pub fn set(&mut self, target: f32, tween: Option<Tween<f32>>) {
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

	pub fn update(&mut self, dt: f32) -> bool {
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
