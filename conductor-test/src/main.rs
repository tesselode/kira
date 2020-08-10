use conductor::{
	manager::AudioManagerSettings,
	metronome::MetronomeSettings,
	sequence::{PlaySoundTaskSettings, Sequence},
	time::Time,
	AudioManager, Project,
};
use std::{error::Error, path::PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
	let mut project = Project::new();
	let sound_id = project.load_sound(&PathBuf::from("whatever.ogg"))?;
	let metronome_id = project.create_metronome(120.0, MetronomeSettings::default());
	let mut audio_manager = AudioManager::new(project, AudioManagerSettings::default())?;
	// create a new sequence that uses a previously created metronome
	let mut sequence = Sequence::new(metronome_id);
	// let's define the steps for the sequence:
	// 1. wait for the next beat
	sequence.on_interval(1.0);
	// 2. play a sound
	let task = sequence.play_sound(sound_id, PlaySoundTaskSettings::default());
	// 3. wait for 4 beats
	sequence.wait(Time::Beats(4.0));
	// 4. stop the sound
	sequence.stop_instance(task, None);
	// 5. go to step 2
	sequence.go_to(1);
	// now that we've defined the sequence, let's start it
	audio_manager.start_sequence(sequence)?;
	Ok(())
}
