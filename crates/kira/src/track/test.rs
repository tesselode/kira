use std::time::Duration;

use crate::{
	clock::clock_info::{ClockInfoProvider, MockClockInfoProviderBuilder},
	effect::{Effect, EffectBuilder},
	frame::Frame,
	modulator::value_provider::{MockModulatorValueProviderBuilder, ModulatorValueProvider},
	track::TrackId,
	tween::Tween,
};

use super::TrackBuilder;

/// Tests that the output volume of a track can be set.
#[test]
fn volume() {
	let mut track = TrackBuilder::new().volume(0.5).build(TrackId::Main).0;
	track.add_input(Frame::from_mono(1.0));
	assert_eq!(
		track.process(
			1.0,
			&MockClockInfoProviderBuilder::new(0).build(),
			&MockModulatorValueProviderBuilder::new(0).build()
		),
		Frame::from_mono(0.5)
	);
}

/// Tests that the output volume of a track can be changed
/// after it's created.
#[test]
fn set_volume() {
	let (mut track, mut handle) = TrackBuilder::new().build(TrackId::Main);
	handle.set_volume(
		0.5,
		Tween {
			duration: Duration::ZERO,
			..Default::default()
		},
	);
	track.on_start_processing();
	track.add_input(Frame::from_mono(1.0));
	assert_eq!(
		track.process(
			1.0,
			&MockClockInfoProviderBuilder::new(0).build(),
			&MockModulatorValueProviderBuilder::new(0).build()
		),
		Frame::from_mono(0.5)
	);
}

/// Tests that effects process the input signal in order.
#[test]
fn effects() {
	let mut track = {
		let mut builder = TrackBuilder::new();
		builder.add_effect(MockEffect::Add(Frame::from_mono(0.5)));
		builder.add_effect(MockEffect::Mul(0.5));
		builder.build(TrackId::Main).0
	};
	track.add_input(Frame::from_mono(1.0));
	assert_eq!(
		track.process(
			1.0,
			&MockClockInfoProviderBuilder::new(0).build(),
			&MockModulatorValueProviderBuilder::new(0).build()
		),
		Frame::from_mono(0.75)
	);
}

enum MockEffect {
	Add(Frame),
	Mul(f32),
}

impl EffectBuilder for MockEffect {
	type Handle = ();

	fn build(self) -> (Box<dyn Effect>, Self::Handle) {
		(Box::new(self), ())
	}
}

impl Effect for MockEffect {
	fn process(
		&mut self,
		input: Frame,
		_dt: f64,
		_clock_info_provider: &ClockInfoProvider,
		_modulator_value_provider: &ModulatorValueProvider,
	) -> Frame {
		match self {
			MockEffect::Add(frame) => input + *frame,
			MockEffect::Mul(amount) => input * *amount,
		}
	}
}
