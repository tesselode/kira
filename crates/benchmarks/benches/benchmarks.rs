use std::{f32::consts::TAU, sync::Arc};

use criterion::{criterion_group, criterion_main, Criterion};
use kira::{
	backend::mock::{MockBackend, MockBackendSettings},
	sound::static_sound::{StaticSoundData, StaticSoundSettings},
	track::MainTrackBuilder,
	AudioManager, AudioManagerSettings, Frame,
};

fn create_test_sound(num_samples: usize) -> StaticSoundData {
	const SAMPLE_RATE: u32 = 48_000;
	let mut frames = vec![];
	let mut phase = 0.0;
	for _ in 0..num_samples {
		frames.push(Frame::from_mono((phase * TAU).sin()));
		phase += 440.0 / SAMPLE_RATE as f32;
	}
	StaticSoundData {
		sample_rate: SAMPLE_RATE,
		frames: Arc::from(frames),
		settings: StaticSoundSettings::new().loop_region(0.0..),
		slice: None,
	}
}

fn sounds(c: &mut Criterion) {
	// a simple test case where many sounds are being played at once
	c.bench_function("simple", |b| {
		const SAMPLE_RATE: u32 = 48_000;
		const NUM_SOUNDS: usize = 5000;
		let mut manager = AudioManager::<MockBackend>::new(AudioManagerSettings {
			backend_settings: MockBackendSettings {
				sample_rate: SAMPLE_RATE,
			},
			main_track_builder: MainTrackBuilder::new().sound_capacity(NUM_SOUNDS),
			..Default::default()
		})
		.unwrap();
		let sound_data = create_test_sound(SAMPLE_RATE as usize);
		for _ in 0..NUM_SOUNDS {
			manager.play(sound_data.clone()).unwrap();
		}
		manager.backend_mut().on_start_processing();
		b.iter(|| manager.backend_mut().process());
	});

	// similar to "simple", but also periodically calls the
	// on_start_processing callback to measure its relative
	// impact on the performance
	c.bench_function("with on_start_processing callback", |b| {
		const SAMPLE_RATE: u32 = 48_000;
		const NUM_SOUNDS: usize = 5000;
		let mut manager = AudioManager::<MockBackend>::new(AudioManagerSettings {
			backend_settings: MockBackendSettings {
				sample_rate: SAMPLE_RATE,
			},
			main_track_builder: MainTrackBuilder::new().sound_capacity(NUM_SOUNDS),
			..Default::default()
		})
		.unwrap();
		let sound_data = create_test_sound(SAMPLE_RATE as usize);
		for _ in 0..NUM_SOUNDS {
			manager.play(sound_data.clone()).unwrap();
		}
		b.iter(|| {
			manager.backend_mut().on_start_processing();
			manager.backend_mut().process();
		});
	});
}

criterion_group!(benches, sounds);
criterion_main!(benches);
