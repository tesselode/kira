use std::{f32::consts::PI, vec};

use criterion::{criterion_group, criterion_main, Criterion, Fun};
use kira::{
	instance::InstanceId,
	manager::{AudioManager, AudioManagerSettings, Backend},
	playable::PlayableSettings,
	sound::Sound,
	Frame,
};

fn create_test_sound(num_samples: usize) -> Sound {
	const SAMPLE_RATE: u32 = 48000;
	let mut sine_samples = vec![];
	let mut phase = 0.0;
	for _ in 0..num_samples {
		sine_samples.push(Frame::from_mono((phase * 2.0 * PI).sin()));
		phase += 440.0 / SAMPLE_RATE as f32;
	}
	Sound::from_frames(
		SAMPLE_RATE,
		sine_samples,
		PlayableSettings {
			cooldown: None,
			default_loop_start: Some(0.0),
			..Default::default()
		},
	)
}

fn create_manager_with_instances(num_instances: usize) -> (AudioManager, Backend, Vec<InstanceId>) {
	let (mut audio_manager, mut backend) =
		AudioManager::new_without_audio_thread(AudioManagerSettings {
			num_instances: num_instances,
			num_commands: num_instances,
			..Default::default()
		})
		.unwrap();
	// add a test sound
	let sound_id = audio_manager.add_sound(create_test_sound(48000)).unwrap();
	backend.process();
	// start a bunch of instances
	let instance_ids = (0..num_instances)
		.map(|_| audio_manager.play(sound_id, Default::default()).unwrap())
		.collect();
	backend.process();
	(audio_manager, backend, instance_ids)
}

fn instances_benchmark(c: &mut Criterion) {
	const NUM_INSTANCES: usize = 100_000;
	c.bench_functions(
		"instances",
		vec![
			Fun::new("no pitch change", |b, _| {
				let (_, mut backend, _) = create_manager_with_instances(NUM_INSTANCES);
				b.iter(|| backend.process());
			}),
			Fun::new("with pitch change", |b, _| {
				let (mut manager, mut backend, instance_ids) =
					create_manager_with_instances(NUM_INSTANCES);
				let mut instance_to_update = 0;
				b.iter(|| {
					manager
						.set_instance_pitch(instance_ids[instance_to_update], 0.5..1.5)
						.unwrap();
					instance_to_update += 1;
					instance_to_update %= NUM_INSTANCES;
					backend.process();
				});
			}),
		],
		(),
	);
}

criterion_group!(benches, instances_benchmark);
criterion_main!(benches);
