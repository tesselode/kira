use crate::tween::Tween;

pub(super) enum Command {
	Set { target: f64, tween: Tween },
}
