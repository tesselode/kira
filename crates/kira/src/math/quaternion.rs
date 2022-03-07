// See here for quaternion math: https://danceswithcode.net/engineeringnotes/quaternions/quaternions.html

use std::ops::Mul;

use super::Vec3;

/// Represents an orientation in 3D space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quaternion {
	/// The w component of the quaternion.
	pub w: f32,
	/// The x component of the quaternion.
	pub x: f32,
	/// The y component of the quaternion.
	pub y: f32,
	/// The z component of the quaternion.
	pub z: f32,
}

impl Quaternion {
	pub(crate) const IDENTITY: Self = Self {
		w: 1.0,
		x: 0.0,
		y: 0.0,
		z: 0.0,
	};

	/// Returns the inverse of this quaternion.
	pub(crate) fn inverted(self) -> Self {
		Self {
			w: self.w,
			x: -self.x,
			y: -self.y,
			z: -self.z,
		}
	}

	/// Uses this quaternion to rotate a point.
	pub(crate) fn rotate_point(&self, point: Vec3) -> Vec3 {
		let p = Quaternion {
			w: 0.0,
			x: point.x,
			y: point.y,
			z: point.z,
		};
		let Quaternion { x, y, z, .. } = self.inverted() * p * *self;
		Vec3 { x, y, z }
	}
}

impl Default for Quaternion {
	fn default() -> Self {
		Self::IDENTITY
	}
}

impl Mul for Quaternion {
	type Output = Self;

	fn mul(self, rhs: Self) -> Self::Output {
		Self {
			w: self.w * rhs.w - self.x * rhs.x - self.y * rhs.y - self.z * rhs.z,
			x: self.w * rhs.x + self.x * rhs.w + self.y * rhs.z - self.z * rhs.y,
			y: self.w * rhs.y - self.x * rhs.z + self.y * rhs.w + self.z * rhs.x,
			z: self.w * rhs.z + self.x * rhs.y - self.y * rhs.x + self.z * rhs.w,
		}
	}
}
