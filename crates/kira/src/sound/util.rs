use crate::{
	parameter::{Parameter, Value},
	tween::Tween,
	Volume,
};

pub fn create_volume_fade_parameter(fade_in_tween: Option<Tween>) -> Parameter<Volume> {
	if let Some(tween) = fade_in_tween {
		let mut tweenable = Parameter::new(
			Value::Fixed(Volume::Decibels(Volume::MIN_DECIBELS)),
			Volume::Decibels(Volume::MIN_DECIBELS),
		);
		tweenable.set(Value::Fixed(Volume::Decibels(0.0)), tween);
		tweenable
	} else {
		Parameter::new(Value::Fixed(Volume::Decibels(0.0)), Volume::Decibels(0.0))
	}
}
