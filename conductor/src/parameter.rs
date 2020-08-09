pub struct Tween {
	pub start: f32,
	pub target: f32,
	pub duration: f32,
	pub progress: f32,
}

impl Tween {
	pub fn new(start: f32, target: f32, duration: f32) -> Self {
		Self {
			start,
			target,
			duration,
			progress: 0.0,
		}
	}

	pub fn update(&mut self, dt: f32) -> (f32, bool) {
		self.progress += dt / self.duration;
		self.progress = self.progress.min(1.0);
		(
			self.start + (self.target - self.start) * self.progress,
			self.progress >= 1.0,
		)
	}
}

pub struct Parameter {
	value: f32,
	tween: Option<Tween>,
}

impl Parameter {
	pub fn new(value: f32) -> Self {
		Self { value, tween: None }
	}

	pub fn value(&self) -> f32 {
		self.value
	}

	pub fn tween(&mut self, target: f32, duration: f32) {
		self.tween = Some(Tween::new(self.value, target, duration));
	}

	pub fn update(&mut self, dt: f32) -> bool {
		if let Some(tween) = &mut self.tween {
			let (value, finished) = tween.update(dt);
			self.value = value;
			if finished {
				self.tween = None;
				return true;
			}
		}
		false
	}
}
