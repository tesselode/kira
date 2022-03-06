use std::time::Duration;

use crate::{dsp::Frame, tween::Tween, Volume};

use super::{
	effect::{Effect, EffectBuilder},
	Track, TrackBuilder,
};

/// Tests that the output volume of a track can be set.
#[test]
fn volume() {
	let mut track = Track::new(TrackBuilder::new().volume(0.5));
	track.add_input(Frame::from_mono(1.0));
	assert_eq!(track.process(1.0), Frame::from_mono(0.5));
}

/// Tests that the output volume of a track can be changed
/// after it's created.
#[test]
fn set_volume() {
	let mut track = Track::new(TrackBuilder::new());
	track.set_volume(
		Volume::Amplitude(0.5),
		Tween {
			duration: Duration::ZERO,
			..Default::default()
		},
	);
	track.add_input(Frame::from_mono(1.0));
	assert_eq!(track.process(1.0), Frame::from_mono(0.5));
}

/// Tests that effects process the input signal in order.
#[test]
fn effects() {
	let mut track = Track::new({
		let mut builder = TrackBuilder::new();
		builder.add_effect(MockEffect::Add(Frame::from_mono(0.5)));
		builder.add_effect(MockEffect::Mul(0.5));
		builder
	});
	track.add_input(Frame::from_mono(1.0));
	assert_eq!(track.process(1.0), Frame::from_mono(0.75));
}

enum MockEffect {
	Add(Frame),
	Mul(f32),
}

impl EffectBuilder for MockEffect {
	type Handle = ();

	fn build(self) -> (Box<dyn super::effect::Effect>, Self::Handle) {
		(Box::new(self), ())
	}
}

impl Effect for MockEffect {
	fn process(&mut self, input: Frame, _dt: f64) -> Frame {
		match self {
			MockEffect::Add(frame) => input + *frame,
			MockEffect::Mul(amount) => input * *amount,
		}
	}
}
