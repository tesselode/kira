use std::f32::consts::TAU;

use criterion::{criterion_group, criterion_main, Criterion};
use kira::{
	manager::{AudioManager, AudioManagerSettings, MockBackend},
	sound::{
		instance::InstanceSettings,
		static_sound::{StaticSound, StaticSoundSettings},
	},
	Frame, LoopBehavior,
};

fn create_test_sound(num_frames: usize) -> StaticSound {
	const SAMPLE_RATE: u32 = 48_000;
	let mut frames = vec![];
	let mut phase = 0.0;
	for _ in 0..num_frames {
		frames.push(Frame::from_mono((phase * TAU).sin()));
		phase += 440.0 / SAMPLE_RATE as f32;
	}
	StaticSound::from_frames(
		SAMPLE_RATE,
		frames,
		StaticSoundSettings::new().default_loop_behavior(LoopBehavior {
			start_position: 0.0,
		}),
	)
}

fn instances_benchmark(c: &mut Criterion) {
	let mut benchmark_group = c.benchmark_group("instances");

	benchmark_group.bench_function("simple", |b| {
		const NUM_INSTANCES: usize = 100_000;
		let mut manager = AudioManager::new(
			AudioManagerSettings {
				command_capacity: NUM_INSTANCES,
				instance_capacity: NUM_INSTANCES,
				..Default::default()
			},
			MockBackend::new(48_000),
		)
		.unwrap();
		let mut sound = manager.add_sound(create_test_sound(48_000)).unwrap();
		manager.backend_mut().on_start_processing(0.0);
		for _ in 0..NUM_INSTANCES {
			sound.play(Default::default()).unwrap();
		}
		manager.backend_mut().on_start_processing(0.0);
		b.iter(|| manager.backend_mut().process());
	});

	benchmark_group.bench_function("with parameters", |b| {
		const NUM_INSTANCES: usize = 100_000;
		let mut manager = AudioManager::new(
			AudioManagerSettings {
				command_capacity: NUM_INSTANCES,
				instance_capacity: NUM_INSTANCES,
				..Default::default()
			},
			MockBackend::new(48_000),
		)
		.unwrap();
		let mut sound = manager.add_sound(create_test_sound(48_000)).unwrap();
		let parameter_1 = manager.add_parameter(1.0).unwrap();
		let parameter_2 = manager.add_parameter(1.0).unwrap();
		let parameter_3 = manager.add_parameter(1.0).unwrap();
		manager.backend_mut().on_start_processing(0.0);
		for _ in 0..NUM_INSTANCES {
			sound
				.play(
					InstanceSettings::new()
						.volume(&parameter_1)
						.playback_rate(&parameter_2)
						.panning(&parameter_3),
				)
				.unwrap();
		}
		manager.backend_mut().on_start_processing(0.0);
		b.iter(|| manager.backend_mut().process());
	});
}

criterion_group!(benches, instances_benchmark);
criterion_main!(benches);
