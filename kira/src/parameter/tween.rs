use atomig::Atom;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Atom)]
#[repr(u8)]
pub enum Easing {
	Linear,
	InQuad,
	OutQuad,
	InOutQuad,
}

impl Easing {
	pub(crate) fn ease(&self, mut x: f64) -> f64 {
		match self {
			Easing::Linear => x,
			Easing::InQuad => x.powi(2),
			Easing::OutQuad => 1.0 - Easing::InQuad.ease(1.0 - x),
			Easing::InOutQuad => {
				x *= 2.0;
				if x < 1.0 {
					0.5 * Easing::InQuad.ease(x)
				} else {
					x = 2.0 - x;
					0.5 * (1.0 - Easing::InQuad.ease(x)) + 0.5
				}
			}
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tween {
	pub duration: f64,
	pub easing: Easing,
}
