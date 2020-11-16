use kira::{
	instance::InstanceSettings,
	manager::AudioManager,
	sequence::Sequence,
	sound::{Sound, SoundMetadata, SoundSettings},
	KiraError, Tempo,
};
use std::error::Error;

#[derive(Debug, Copy, Clone)]
enum CustomEvent {
	KickDrum,
}

fn main() -> Result<(), KiraError> {
	let mut audio_manager = AudioManager::<CustomEvent>::new(Default::default())?;
	let kick_drum_sound_id = audio_manager.add_sound(Sound::from_file(
		"loop.ogg",
		SoundSettings {
			metadata: SoundMetadata {
				semantic_duration: Some(Tempo(128.0).beats_to_seconds(16.0)),
			},
			..Default::default()
		},
	)?)?;
	let mut sequence = Sequence::new();
	sequence.start_loop();
	sequence.wait_for_interval(4.0);
	sequence.play_sound(kick_drum_sound_id, Default::default());
	sequence.emit_custom_event(CustomEvent::KickDrum);
	audio_manager.start_sequence(sequence)?;
	// start the metronome so the sequence will have a pulse to listen for
	audio_manager.start_metronome()?;
	Ok::<(), kira::KiraError>(())
}
