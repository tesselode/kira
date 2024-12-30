use std::{error::Error, io::stdin, time::Duration};

use kira::{
	backend::DefaultBackend,
	effect::{filter::FilterBuilder, reverb::ReverbBuilder},
	modulator::tweener::TweenerBuilder,
	sound::static_sound::{StaticSoundData, StaticSoundSettings},
	track::TrackBuilder,
	tween::{Easing, Tween},
	AudioManager, AudioManagerSettings, Decibels, Mapping, Mix, Value,
};

fn main() -> Result<(), Box<dyn Error>> {
	let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
	// create the tweener that will be used to change multiple parameters at once
	let mut underwater_tweener = manager.add_modulator(TweenerBuilder { initial_value: 0.0 })?;
	// create a mixer track for the lead instrument. this has a filter and reverb that become
	// more prominent as the tweener value reaches 1.0.
	let mut lead_track = manager.add_sub_track(
		TrackBuilder::new()
			.with_effect(FilterBuilder::new().cutoff(Value::from_modulator(
				&underwater_tweener,
				Mapping {
					input_range: (0.0, 1.0),
					output_range: (20_000.0, 2000.0),
					easing: Easing::Linear,
				},
			)))
			.with_effect(ReverbBuilder::new().mix(Value::from_modulator(
				&underwater_tweener,
				Mapping {
					input_range: (0.0, 1.0),
					output_range: (Mix::DRY, Mix(1.0 / 3.0)),
					easing: Easing::Linear,
				},
			))),
	)?;
	// set a loop region (used for all the sounds, since they're the same length)
	let music_duration = 21.0 + 1.0 / 3.0;
	let common_sound_settings =
		StaticSoundSettings::new().loop_region(music_duration / 2.0..music_duration);
	// load the sounds, linking the volumes to the tweener when appropriate
	let arp = StaticSoundData::from_file("crates/examples/assets/dynamic/arp.ogg")?
		.with_settings(common_sound_settings);
	let bass = StaticSoundData::from_file("crates/examples/assets/dynamic/bass.ogg")?
		.with_settings(common_sound_settings)
		.volume(Value::from_modulator(
			&underwater_tweener,
			Mapping {
				input_range: (0.0, 1.0),
				output_range: (Decibels::IDENTITY, Decibels::SILENCE),
				easing: Easing::Linear,
			},
		));
	let drums = StaticSoundData::from_file("crates/examples/assets/dynamic/drums.ogg")?
		.with_settings(common_sound_settings)
		.volume(Value::from_modulator(
			&underwater_tweener,
			Mapping {
				input_range: (0.0, 1.0),
				output_range: (Decibels::IDENTITY, Decibels::SILENCE),
				easing: Easing::Linear,
			},
		));
	let lead = StaticSoundData::from_file("crates/examples/assets/dynamic/lead.ogg")?
		.with_settings(common_sound_settings);
	let pad = StaticSoundData::from_file("crates/examples/assets/dynamic/pad.ogg")?
		.with_settings(common_sound_settings)
		.volume(Value::from_modulator(
			&underwater_tweener,
			Mapping {
				input_range: (0.0, 1.0),
				output_range: (Decibels::SILENCE, Decibels::IDENTITY),
				easing: Easing::Linear,
			},
		));
	// play the sounds
	manager.play(arp)?;
	manager.play(bass)?;
	manager.play(drums)?;
	lead_track.play(lead)?;
	manager.play(pad)?;

	println!("Press enter to switch music variations");
	let mut underwater = false;
	loop {
		wait_for_enter_press()?;
		underwater = !underwater;
		underwater_tweener.set(
			if underwater { 1.0 } else { 0.0 },
			Tween {
				duration: Duration::from_secs(3),
				..Default::default()
			},
		);
		if underwater {
			println!("submerging...");
		} else {
			println!("resurfacing...");
		}
	}
}

fn wait_for_enter_press() -> Result<(), Box<dyn Error>> {
	stdin().read_line(&mut "".into())?;
	Ok(())
}
