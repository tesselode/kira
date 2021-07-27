use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Easing {
	Linear,
	InPowi(i32),
	OutPowi(i32),
	InOutPowi(i32),
	InPowf(f64),
	OutPowf(f64),
	InOutPowf(f64),
}

impl Easing {
	fn apply(&self, mut x: f64) -> f64 {
		match self {
			Easing::Linear => x,
			Easing::InPowi(power) => x.powi(*power),
			Easing::OutPowi(power) => 1.0 - Self::InPowi(*power).apply(1.0 - x),
			Easing::InOutPowi(power) => {
				x *= 2.0;
				if x < 1.0 {
					0.5 * Self::InPowi(*power).apply(x)
				} else {
					x = 2.0 - x;
					0.5 * (1.0 - Self::InPowi(*power).apply(x)) + 0.5
				}
			}
			Easing::InPowf(power) => x.powf(*power),
			Easing::OutPowf(power) => 1.0 - Self::InPowf(*power).apply(1.0 - x),
			Easing::InOutPowf(power) => {
				x *= 2.0;
				if x < 1.0 {
					0.5 * Self::InPowf(*power).apply(x)
				} else {
					x = 2.0 - x;
					0.5 * (1.0 - Self::InPowf(*power).apply(x)) + 0.5
				}
			}
		}
	}
}

impl Default for Easing {
	fn default() -> Self {
		Self::Linear
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Tween {
	pub delay: Duration,
	pub duration: Duration,
	pub easing: Easing,
}

impl Tween {
	fn value(&self, mut time: f64) -> f64 {
		time -= self.delay.as_secs_f64();
		if time < 0.0 {
			return 0.0;
		}
		self.easing.apply(time / self.duration.as_secs_f64())
	}
}
