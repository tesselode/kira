use std::{error::Error, io::stdin, time::Duration};

use kira::{
	manager::{backend::cpal::CpalBackend, AudioManager, AudioManagerSettings},
	sound::static_sound::{StaticSoundData, StaticSoundSettings},
	track::{effect::reverb::ReverbBuilder, TrackBuilder, TrackRoutes},
	tween::Tween,
	LoopBehavior,
};

fn main() -> Result<(), Box<dyn Error>> {
	let mut manager = AudioManager::<CpalBackend>::new(AudioManagerSettings::default())?;

	// create a mixer track with a reverb effect on it. setting
	// the `mix` to 1.0 means you'll only hear reverb coming out
	// of this track, not the original audio.
	let reverb = manager.add_sub_track({
		let mut builder = TrackBuilder::new();
		builder.add_effect(ReverbBuilder::new().mix(1.0));
		builder
	})?;
	// create a track for playing sound effects. this track is routed
	// to both the main track and the reverb track, so the final result
	// will be the original signal and the reverb signal added together.
	let sfx = manager
		.add_sub_track(TrackBuilder::new().routes(TrackRoutes::new().with_route(&reverb, 0.5)))?;
	let sound = StaticSoundData::from_file(
		"crates/examples/assets/blip.ogg",
		StaticSoundSettings::new()
			.loop_behavior(LoopBehavior {
				start_position: 0.0,
			})
			.output_destination(&sfx),
	)?;
	manager.play(sound)?;

	println!("Press enter to toggle reverb");
	let mut reverb_enabled = true;
	loop {
		stdin().read_line(&mut "".into())?;
		reverb_enabled = !reverb_enabled;
		// smoothly adjust the amount of signal from the sfx track
		// that's routed to the reverb track
		sfx.set_route(
			&reverb,
			if reverb_enabled { 0.5 } else { 0.0 },
			Tween {
				duration: Duration::from_secs(1),
				..Default::default()
			},
		)?;
	}
}
