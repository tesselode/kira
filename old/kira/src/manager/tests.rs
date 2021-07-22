use crate::{arrangement::Arrangement, sound::Sound};

use super::{
	error::{
		AddArrangementError, AddGroupError, AddMetronomeError, AddParameterError,
		AddSendTrackError, AddSoundError, AddSubTrackError,
	},
	AudioManager, AudioManagerSettings,
};

fn create_manager_with_limited_capacity() -> AudioManager {
	let (manager, _) = AudioManager::new_without_audio_thread(AudioManagerSettings {
		num_sounds: 1,
		num_arrangements: 1,
		num_parameters: 1,
		num_instances: 1,
		num_sequences: 1,
		num_sub_tracks: 1,
		num_send_tracks: 1,
		num_groups: 1,
		num_streams: 1,
		num_metronomes: 1,
		..Default::default()
	});
	manager
}

#[test]
fn returns_error_on_exceeded_sound_capacity() {
	let mut manager = create_manager_with_limited_capacity();
	let sound = Sound::from_frames(48000, vec![], Default::default());
	assert!(manager.add_sound(sound.clone()).is_ok());
	if let Err(AddSoundError::SoundLimitReached) = manager.add_sound(sound.clone()) {
	} else {
		panic!("AudioManager::add_sound should return Err(AddSoundError::SoundLimitReached) when the maximum number of sounds is exceeded");
	}
}

#[test]
fn returns_error_on_exceeded_arrangement_capacity() {
	let mut manager = create_manager_with_limited_capacity();
	let arrangement = Arrangement::new(Default::default());
	assert!(manager.add_arrangement(arrangement.clone()).is_ok());
	if let Err(AddArrangementError::ArrangementLimitReached) =
		manager.add_arrangement(arrangement.clone())
	{
	} else {
		panic!("AudioManager::add_arrangement should return Err(AddArrangementError::ArrangementLimitReached) when the maximum number of arrangements is exceeded");
	}
}

#[test]
fn returns_error_on_exceeded_parameter_capacity() {
	let mut manager = create_manager_with_limited_capacity();
	assert!(manager.add_parameter(Default::default()).is_ok());
	if let Err(AddParameterError::ParameterLimitReached) = manager.add_parameter(Default::default())
	{
	} else {
		panic!("AudioManager::add_parameter should return Err(AddParameterError::ParameterLimitReached) when the maximum number of arrangements is exceeded");
	}
}

#[test]
fn returns_error_on_exceeded_sub_track_capacity() {
	let mut manager = create_manager_with_limited_capacity();
	assert!(manager.add_sub_track(Default::default()).is_ok());
	if let Err(AddSubTrackError::TrackLimitReached) = manager.add_sub_track(Default::default()) {
	} else {
		panic!("AudioManager::add_sub_track should return Err(AddSubTrackError::TrackLimitReached) when the maximum number of arrangements is exceeded");
	}
}

#[test]
fn returns_error_on_exceeded_send_track_capacity() {
	let mut manager = create_manager_with_limited_capacity();
	assert!(manager.add_send_track(Default::default()).is_ok());
	if let Err(AddSendTrackError::TrackLimitReached) = manager.add_send_track(Default::default()) {
	} else {
		panic!("AudioManager::add_send_track should return Err(AddSendTrackError::TrackLimitReached) when the maximum number of arrangements is exceeded");
	}
}

#[test]
fn returns_error_on_exceeded_group_capacity() {
	let mut manager = create_manager_with_limited_capacity();
	assert!(manager.add_group(Default::default()).is_ok());
	if let Err(AddGroupError::GroupLimitReached) = manager.add_group(Default::default()) {
	} else {
		panic!("AudioManager::add_group should return Err(AddGroupError::GroupLimitReached) when the maximum number of arrangements is exceeded");
	}
}

#[test]
fn returns_error_on_exceeded_metronome_capacity() {
	let mut manager = create_manager_with_limited_capacity();
	assert!(manager.add_metronome(Default::default()).is_ok());
	if let Err(AddMetronomeError::MetronomeLimitReached) = manager.add_metronome(Default::default())
	{
	} else {
		panic!("AudioManager::add_metronome should return Err(AddMetronomeError::MetronomeLimitReached) when the maximum number of arrangements is exceeded");
	}
}

// TODO: write a test for exceeded stream capacity
