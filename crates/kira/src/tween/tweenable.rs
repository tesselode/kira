use glam::{Quat, Vec3};

/// A trait for types that can be used with a [`Tweener`](super::Tweener).
pub trait Tweenable: Copy {
	/// Returns an linearly interpolated value between `a` and `b`.
	///
	/// An amount of `0.0` should yield `a`, an amount of `1.0` should
	/// yield `b`, and an amount of `0.5` should yield a value halfway
	/// between `a` and `b`.
	fn interpolate(a: Self, b: Self, amount: f64) -> Self;
}

impl Tweenable for f64 {
	fn interpolate(a: Self, b: Self, amount: f64) -> Self {
		a + (b - a) * amount
	}
}

impl Tweenable for Vec3 {
	fn interpolate(a: Self, b: Self, amount: f64) -> Self {
		a + (b - a) * amount as f32
	}
}

impl Tweenable for Quat {
	fn interpolate(a: Self, b: Self, amount: f64) -> Self {
		a.slerp(b, amount as f32)
	}
}
