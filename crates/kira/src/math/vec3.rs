use std::ops::{Add, AddAssign, Sub, SubAssign};

/// A 3-dimensional vector.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec3 {
	/// The x component of the vector.
	pub x: f32,
	/// The y component of the vector.
	pub y: f32,
	/// The z component of the vector.
	pub z: f32,
}

impl Vec3 {
	pub fn magnitude_squared(&self) -> f32 {
		self.x.powi(2) + self.y.powi(2) + self.z.powi(2)
	}

	pub fn magnitude(&self) -> f32 {
		self.magnitude_squared().sqrt()
	}
}

impl Add for Vec3 {
	type Output = Vec3;

	fn add(self, rhs: Self) -> Self::Output {
		Self {
			x: self.x + rhs.x,
			y: self.y + rhs.y,
			z: self.z + rhs.z,
		}
	}
}

impl AddAssign for Vec3 {
	fn add_assign(&mut self, rhs: Self) {
		self.x += rhs.x;
		self.y += rhs.y;
		self.z += rhs.z;
	}
}

impl Sub for Vec3 {
	type Output = Vec3;

	fn sub(self, rhs: Self) -> Self::Output {
		Self {
			x: self.x - rhs.x,
			y: self.y - rhs.y,
			z: self.z - rhs.z,
		}
	}
}

impl SubAssign for Vec3 {
	fn sub_assign(&mut self, rhs: Self) {
		self.x -= rhs.x;
		self.y -= rhs.y;
		self.z -= rhs.z;
	}
}
