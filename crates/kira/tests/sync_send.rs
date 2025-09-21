use kira::{
	AudioManager,
	backend::cpal::CpalBackend,
	clock::ClockHandle,
	effect::{
		compressor::CompressorHandle, delay::DelayHandle, distortion::DistortionHandle,
		eq_filter::EqFilterHandle, filter::FilterHandle, panning_control::PanningControlHandle,
		reverb::ReverbHandle, volume_control::VolumeControlHandle,
	},
	listener::ListenerHandle,
	modulator::{lfo::LfoHandle, tweener::TweenerHandle},
	sound::{FromFileError, static_sound::StaticSoundHandle, streaming::StreamingSoundHandle},
	track::{MainTrackHandle, SendTrackHandle, SpatialTrackHandle, TrackHandle},
};

fn main() {
	sync_send::<AudioManager<CpalBackend>>();
	sync_send::<LfoHandle>();
	sync_send::<ClockHandle>();
	sync_send::<DelayHandle>();
	sync_send::<TrackHandle>();
	sync_send::<FilterHandle>();
	sync_send::<ReverbHandle>();
	sync_send::<TweenerHandle>();
	sync_send::<EqFilterHandle>();
	sync_send::<ListenerHandle>();
	sync_send::<MainTrackHandle>();
	sync_send::<SendTrackHandle>();
	sync_send::<CompressorHandle>();
	sync_send::<DistortionHandle>();
	sync_send::<StaticSoundHandle>();
	sync_send::<SpatialTrackHandle>();
	sync_send::<VolumeControlHandle>();
	sync_send::<PanningControlHandle>();
	sync_send::<StreamingSoundHandle<FromFileError>>();
}

fn sync_send<T: Sync + Send>() {}
