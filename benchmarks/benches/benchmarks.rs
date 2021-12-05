use std::{f32::consts::TAU, sync::Arc};

use criterion::{criterion_group, criterion_main, Criterion};
use kira::{
	dsp::Frame,
	manager::{backend::MockBackend, AudioManager, AudioManagerSettings},
	sound::static_sound::StaticSoundData,
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
		frames: Arc::new(frames),
		settings: Default::default(),
	}
}

fn sounds(c: &mut Criterion) {
	c.bench_function("simple", |b| {
		const SAMPLE_RATE: u32 = 48_000;
		const NUM_SOUNDS: usize = 100_000;
		let mut manager = AudioManager::new(
			MockBackend::new(SAMPLE_RATE),
			AudioManagerSettings::new()
				.sound_capacity(NUM_SOUNDS)
				.command_capacity(NUM_SOUNDS),
		)
		.unwrap();
		let sound_data = create_test_sound(SAMPLE_RATE as usize);
		for _ in 0..NUM_SOUNDS {
			manager.play(sound_data.clone()).unwrap();
		}
		manager.backend_mut().on_start_processing();
		b.iter(|| manager.backend_mut().process());
	});
}

criterion_group!(benches, sounds);
criterion_main!(benches);
