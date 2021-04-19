use std::{f32::consts::PI, vec};

use criterion::{criterion_group, criterion_main, Criterion};
use kira::{
	manager::{AudioManager, AudioManagerSettings},
	sound::{
		data::static_sound::StaticSoundData, instance::settings::InstanceSettings,
		settings::SoundSettings,
	},
	Frame,
};

fn create_test_sound_data(num_samples: usize) -> StaticSoundData {
	const SAMPLE_RATE: u32 = 48000;
	let mut sine_samples = vec![];
	let mut phase = 0.0;
	for _ in 0..num_samples {
		sine_samples.push(Frame::from_mono((phase * 2.0 * PI).sin()));
		phase += 440.0 / SAMPLE_RATE as f32;
	}
	StaticSoundData::from_frames(SAMPLE_RATE, sine_samples)
}

fn instances_benchmark(c: &mut Criterion) {
	let mut benchmark_group = c.benchmark_group("instances");

	benchmark_group.bench_function("one sound", |b| {
		const NUM_INSTANCES: usize = 100_000;
		let sound_data = create_test_sound_data(440);
		let (mut audio_manager, mut backend) =
			AudioManager::new_without_audio_thread(AudioManagerSettings {
				num_instances: NUM_INSTANCES,
				num_commands: NUM_INSTANCES,
				..Default::default()
			});
		// start a bunch of instances
		let sound = audio_manager
			.add_sound(sound_data, SoundSettings::new().loop_start(0.0))
			.unwrap();
		backend.process();
		for _ in 0..NUM_INSTANCES {
			audio_manager.play(&sound, InstanceSettings::new()).unwrap();
		}
		backend.process();
		b.iter(|| backend.process());
		drop(backend);
		drop(audio_manager);
	});
}

criterion_group!(benches, instances_benchmark);
criterion_main!(benches);
