use std::sync::{
	atomic::{AtomicU32, Ordering},
	Arc,
};

use kira::{
	clock::clock_info::ClockInfoProvider,
	dsp::Frame,
	manager::{
		backend::mock::{MockBackend, MockBackendSettings},
		AudioManager, AudioManagerSettings,
	},
	track::{
		effect::{Effect, EffectBuilder},
		TrackBuilder,
	},
};
use ringbuf::{Consumer, Producer, RingBuffer};

struct TestEffect {
	sample_rate: Arc<AtomicU32>,
	dt_producer: Producer<f64>,
}

impl Effect for TestEffect {
	fn init(&mut self, sample_rate: u32) {
		self.sample_rate.store(sample_rate, Ordering::SeqCst);
	}

	fn on_change_sample_rate(&mut self, sample_rate: u32) {
		self.sample_rate.store(sample_rate, Ordering::SeqCst);
	}

	fn process(
		&mut self,
		_input: Frame,
		dt: f64,
		_clock_info_provider: &ClockInfoProvider,
	) -> Frame {
		self.dt_producer.push(dt).unwrap();
		Frame::ZERO
	}
}

struct TestEffectHandle {
	sample_rate: Arc<AtomicU32>,
	dt_consumer: Consumer<f64>,
}

struct TestEffectBuilder;

impl EffectBuilder for TestEffectBuilder {
	type Handle = TestEffectHandle;

	fn build(self) -> (Box<dyn Effect>, Self::Handle) {
		let (dt_producer, dt_consumer) = RingBuffer::new(100).split();
		let sample_rate = Arc::new(AtomicU32::new(0));
		(
			Box::new(TestEffect {
				sample_rate: sample_rate.clone(),
				dt_producer,
			}),
			TestEffectHandle {
				sample_rate,
				dt_consumer,
			},
		)
	}
}

#[test]
fn change_sample_rate() {
	let mut manager = AudioManager::<MockBackend>::new(AudioManagerSettings {
		backend_settings: MockBackendSettings { sample_rate: 100 },
		..Default::default()
	})
	.unwrap();
	let mut effect_handle;
	manager
		.add_sub_track({
			let mut builder = TrackBuilder::new();
			effect_handle = builder.add_effect(TestEffectBuilder);
			builder
		})
		.unwrap();
	let backend = manager.backend_mut();
	backend.on_start_processing();
	assert_eq!(effect_handle.sample_rate.load(Ordering::SeqCst), 100);
	backend.process();
	assert_eq!(effect_handle.dt_consumer.pop(), Some(1.0 / 100.0));
	backend.set_sample_rate(200);
	assert_eq!(effect_handle.sample_rate.load(Ordering::SeqCst), 200);
	backend.process();
	assert_eq!(effect_handle.dt_consumer.pop(), Some(1.0 / 200.0));
}
