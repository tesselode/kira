use kira::{
    manager::{AudioManager, AudioManagerSettings},
    sound::static_sound::{PlaybackState, StaticSoundSettings},
};

use kira_cpal::CpalBackend;

use std::thread::sleep;
use std::time::Duration;
use std::path::Path;

const FILENAME: &str = "examples/res/sound.ogg";

fn play_sound(filename: &Path) -> Result<(), Box<dyn std::error::Error>>{
    // Create an audio manager. This plays sounds and manages resources.
    println!("Setting up audio manager");
    let mut audio_manager = AudioManager::new(
            CpalBackend::new()?,
            AudioManagerSettings::default(),
        )?;

    println!("Loading sound");
    let sound_data = kira_loaders::load(filename, StaticSoundSettings::default())?;
    println!("Playing sound");
    let instance = audio_manager.play(sound_data)?;

    while instance.state() != PlaybackState::Stopped
    {
        sleep(Duration::from_millis(100));
    }

    Ok(())
}

fn main() {
    let filepath = Path::new(FILENAME);
    let result = play_sound(filepath);
    match result {
        Ok(()) => {println!("Done");},
        _ => {println!("Something happened")},
    }
}

