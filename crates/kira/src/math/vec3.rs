use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

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
	pub(crate) const LEFT: Self = Self {
		x: -1.0,
		y: 0.0,
		z: 0.0,
	};

	pub(crate) const RIGHT: Self = Self {
		x: 1.0,
		y: 0.0,
		z: 0.0,
	};

	pub fn new(x: f32, y: f32, z: f32) -> Self {
		Self { x, y, z }
	}

	pub(crate) fn magnitude_squared(&self) -> f32 {
		self.x.powi(2) + self.y.powi(2) + self.z.powi(2)
	}

	pub(crate) fn magnitude(&self) -> f32 {
		self.magnitude_squared().sqrt()
	}

	pub(crate) fn normalized(self) -> Self {
		self / self.magnitude()
	}

	pub(crate) fn dot(self, rhs: Vec3) -> f32 {
		self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
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

impl Mul<f32> for Vec3 {
	type Output = Vec3;

	fn mul(self, rhs: f32) -> Self::Output {
		Self {
			x: self.x * rhs,
			y: self.y * rhs,
			z: self.z * rhs,
		}
	}
}

impl MulAssign<f32> for Vec3 {
	fn mul_assign(&mut self, rhs: f32) {
		self.x *= rhs;
		self.y *= rhs;
		self.z *= rhs;
	}
}

impl Div<f32> for Vec3 {
	type Output = Vec3;

	fn div(self, rhs: f32) -> Self::Output {
		Self {
			x: self.x / rhs,
			y: self.y / rhs,
			z: self.z / rhs,
		}
	}
}

impl DivAssign<f32> for Vec3 {
	fn div_assign(&mut self, rhs: f32) {
		self.x /= rhs;
		self.y /= rhs;
		self.z /= rhs;
	}
}
