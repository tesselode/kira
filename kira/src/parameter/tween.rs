/// A curve that can be applied to a [`Tween`].
///
/// Given a position in a tween `t` from 0-1, each easing
/// function transforms `t` differently.
#[derive(Debug, Copy, Clone)]
#[cfg_attr(
	feature = "serde_support",
	derive(serde::Serialize, serde::Deserialize)
)]
pub enum Easing {
	/// Applies no transformation.
	Linear,
	/// Raises `t` to an integer power.
	PowI(i32),
	/// Raises `t` to a float power.
	PowF(f64),
}

impl Easing {
	/// Applies the easing curve to the given `t`.
	pub fn apply(&self, t: f64) -> f64 {
		match self {
			Easing::Linear => t,
			Easing::PowI(power) => t.powi(*power),
			Easing::PowF(power) => t.powf(*power),
		}
	}
}

impl Default for Easing {
	fn default() -> Self {
		Self::Linear
	}
}

/// A direction for an easing curve.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(
	feature = "serde_support",
	derive(serde::Serialize, serde::Deserialize)
)]
pub enum EaseDirection {
	In,
	Out,
	InOut,
}

impl Default for EaseDirection {
	fn default() -> Self {
		Self::In
	}
}

/// A movement of one value to another over time.
#[derive(Debug, Copy, Clone)]
#[cfg_attr(
	feature = "serde_support",
	derive(serde::Serialize, serde::Deserialize)
)]
pub struct Tween {
	pub duration: f64,
	pub easing: Easing,
	pub ease_direction: EaseDirection,
}

impl Tween {
	/// Creates a linear tween with the specified duration.
	pub fn linear(duration: f64) -> Self {
		Self {
			duration,
			easing: Default::default(),
			ease_direction: Default::default(),
		}
	}

	/// Applies the tween's easing curve (with easing direction)
	/// to a relative position in an animation (where 0 is the
	/// beginning of the animation and 1 is the end).
	pub fn ease(&self, mut t: f64) -> f64 {
		/* the code for applying In/Out/InOut directions
		to an easing function is based on rxi's flux:
		https://github.com/rxi/flux/blob/master/flux.lua#L33 */
		match self.ease_direction {
			EaseDirection::In => self.easing.apply(t),
			EaseDirection::Out => 1.0 - self.easing.apply(1.0 - t),
			EaseDirection::InOut => {
				t *= 2.0;
				if t < 1.0 {
					0.5 * self.easing.apply(t)
				} else {
					t = 2.0 - t;
					0.5 * (1.0 - self.easing.apply(t)) + 0.5
				}
			}
		}
	}

	/// Gets the value of an animation from one point to another
	/// at the given time (with this tween's duration and easing curve).
	pub fn tween(&self, from: f64, to: f64, time: f64) -> f64 {
		// get the time in the animation relative to the duration
		// of the animation (0 = beginning, 1 = end)
		let mut t = time / self.duration;
		// apply the easing curve
		t = self.ease(t);
		// use a simple lerp to get the resulting value
		from + (to - from) * t
	}
}

impl From<f64> for Tween {
	fn from(duration: f64) -> Self {
		Self {
			duration,
			easing: Easing::default(),
			ease_direction: EaseDirection::default(),
		}
	}
}
