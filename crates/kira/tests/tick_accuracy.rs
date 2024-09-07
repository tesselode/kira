/// This test checks, if the clock ticks are frame-accurate.
/// It does so by starting a clock with given `TICKS_PER_SECOND` and running `backend.on_start_processing()` at a number of frames that isn't a divider of `TICKS_PER_SECOND`.
/// This is to prevent regressions, such as fixed by commit 2c596af.
///
/// Then an effect is added that counts the number of processed frames and checks every `EVENT_TIME_TICKS` (by calling `clock_info_provider.when_to_start`), if the counted number of frames is correct.
/// Here, one has to take care that `TICKS_PER_SECOND` is divisible by `EVENT_TIME_TICKS` and `SAMPLE_RATE` is divisible by `TICKS_PER_SECOND / EVENT_TIME_TICKS`.
/// Otherwise the test is incorrect.
///
/// In the current setting we expect a tick every 24000 frames.
use kira::{
	clock::{ClockHandle, ClockSpeed, ClockTime},
	effect::{Effect, EffectBuilder},
	info::{Info, WhenToStart},
	manager::{
		backend::mock::{MockBackend, MockBackendSettings},
		AudioManager, AudioManagerSettings,
	},
	track::TrackBuilder,
	Frame,
};

const SAMPLE_RATE: u32 = 48_000;
const TICKS_PER_SECOND: f64 = 1000.0;
const EVENT_TIME_TICKS: u64 = 500;

struct TestEffect {
	clock: ClockHandle,
	ticks: u64,
	frames: u32,
}

impl Effect for TestEffect {
	fn process(&mut self, input: Frame, _dt: f64, info: &Info) -> Frame {
		self.frames += 1;
		if self.frames == 24000 {
			println!("asdf");
		}
		if let WhenToStart::Now = info.when_to_start(ClockTime {
			clock: self.clock.id(),
			ticks: self.ticks,
			fraction: 0.0,
		}) {
			assert_eq!(
				self.frames % (SAMPLE_RATE / (TICKS_PER_SECOND as u32 / EVENT_TIME_TICKS as u32)),
				0
			);
			self.ticks += EVENT_TIME_TICKS;
		}
		input
	}
}

struct TestEffectHandle;

struct TestEffectBuilder {
	clock: ClockHandle,
}

impl EffectBuilder for TestEffectBuilder {
	type Handle = TestEffectHandle;

	fn build(self) -> (Box<dyn Effect>, Self::Handle) {
		(
			Box::new(TestEffect {
				clock: self.clock,
				ticks: EVENT_TIME_TICKS,
				frames: 0,
			}),
			TestEffectHandle {},
		)
	}
}

#[test]
fn tick_accuracy() {
	let mut manager = AudioManager::<MockBackend>::new(AudioManagerSettings {
		backend_settings: MockBackendSettings {
			sample_rate: SAMPLE_RATE,
		},
		..Default::default()
	})
	.unwrap();

	let mut clock = manager
		.add_clock(ClockSpeed::TicksPerSecond(TICKS_PER_SECOND))
		.unwrap();
	clock.start();

	let _effect_handle;
	let _track = manager
		.add_sub_track({
			let mut builder = TrackBuilder::new();
			_effect_handle = builder.add_effect(TestEffectBuilder { clock });
			builder
		})
		.unwrap();

	let backend = manager.backend_mut();
	for i in 0..480000 {
		if i % 512 == 0 {
			backend.on_start_processing();
		}
		let _ = backend.process();
	}
}
