use crate::{
	tween::{Parameter, Tween, Value},
	Decibels,
};

pub fn create_volume_fade_parameter(fade_in_tween: Option<Tween>) -> Parameter<Decibels> {
	if let Some(tween) = fade_in_tween {
		let mut tweenable = Parameter::new(Value::Fixed(Decibels::MIN), Decibels::MIN);
		tweenable.set(Value::Fixed(Decibels(0.0)), tween);
		tweenable
	} else {
		Parameter::new(Value::Fixed(Decibels(0.0)), Decibels(0.0))
	}
}
