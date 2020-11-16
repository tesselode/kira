//! Provides an interface to work with pieces of audio.

mod id;
mod metadata;

pub use id::SoundId;
pub use metadata::SoundMetadata;

use crate::{error::KiraError, error::KiraResult, mixer::TrackIndex, stereo_sample::StereoSample};
use claxon::FlacReader;
use hound::WavReader;
use lewton::{inside_ogg::OggStreamReader, samples::Samples};
use std::{fs::File, path::Path};

/// Settings for a sound.
#[derive(Debug, Clone)]
pub struct SoundSettings {
	/// The track instances of this sound will play on by default.
	pub default_track: TrackIndex,
	/// Whether the sound should have a "cool off" period after playing
	/// before it can be played again, and if so, the duration
	/// of that cool off period.
	///
	/// This is useful to avoid situations where the same sound
	/// is played multiple times at the exact same point in time,
	/// resulting in the sound being louder than normal.
	pub cooldown: Option<f64>,
	/// Information about the sound.
	pub metadata: SoundMetadata,
}

impl Default for SoundSettings {
	fn default() -> Self {
		Self {
			default_track: TrackIndex::Main,
			cooldown: Some(0.0001),
			metadata: SoundMetadata::default(),
		}
	}
}

/// A piece of audio that can be played by an `AudioManager`.
#[derive(Debug)]
pub struct Sound {
	sample_rate: u32,
	samples: Vec<StereoSample>,
	duration: f64,
	default_track: TrackIndex,
	cooldown: Option<f64>,
	metadata: SoundMetadata,
	cooldown_timer: f64,
}

impl Sound {
	/// Creates a new sound from raw sample data.
	pub fn new(sample_rate: u32, samples: Vec<StereoSample>, settings: SoundSettings) -> Self {
		let duration = samples.len() as f64 / sample_rate as f64;
		Self {
			sample_rate,
			samples,
			duration,
			default_track: settings.default_track,
			cooldown: settings.cooldown,
			metadata: settings.metadata,
			cooldown_timer: 0.0,
		}
	}

	/// Decodes a sound from an mp3 file.
	pub fn from_mp3_file<P>(path: P, settings: SoundSettings) -> KiraResult<Self>
	where
		P: AsRef<Path>,
	{
		let mut decoder = minimp3::Decoder::new(File::open(path)?);
		let mut sample_rate = None;
		let mut stereo_samples = vec![];
		loop {
			match decoder.next_frame() {
				Ok(frame) => {
					if let Some(sample_rate) = sample_rate {
						if sample_rate != frame.sample_rate {
							return Err(KiraError::VariableMp3SampleRate);
						}
					} else {
						sample_rate = Some(frame.sample_rate);
					}
					match frame.channels {
						1 => {
							for sample in frame.data {
								stereo_samples.push(StereoSample::from_i32(
									sample.into(),
									sample.into(),
									16,
								))
							}
						}
						2 => {
							let mut iter = frame.data.iter();
							while let (Some(left), Some(right)) = (iter.next(), iter.next()) {
								stereo_samples.push(StereoSample::from_i32(
									(*left).into(),
									(*right).into(),
									16,
								))
							}
						}
						_ => return Err(KiraError::UnsupportedChannelConfiguration),
					}
				}
				Err(error) => match error {
					minimp3::Error::Eof => break,
					error => return Err(error.into()),
				},
			}
		}
		let sample_rate = match sample_rate {
			Some(sample_rate) => sample_rate,
			None => return Err(KiraError::UnknownMp3SampleRate),
		};
		Ok(Self::new(sample_rate as u32, stereo_samples, settings))
	}

	/// Decodes a sound from an ogg file.
	pub fn from_ogg_file<P>(path: P, settings: SoundSettings) -> KiraResult<Self>
	where
		P: AsRef<Path>,
	{
		let mut reader = OggStreamReader::new(File::open(path)?)?;
		let mut stereo_samples = vec![];
		while let Some(packet) = reader.read_dec_packet_generic::<Vec<Vec<f32>>>()? {
			let num_channels = packet.len();
			let num_samples = packet.num_samples();
			match num_channels {
				1 => {
					for i in 0..num_samples {
						stereo_samples.push(StereoSample::from_mono(packet[0][i]));
					}
				}
				2 => {
					for i in 0..num_samples {
						stereo_samples.push(StereoSample::new(packet[0][i], packet[1][i]));
					}
				}
				_ => return Err(KiraError::UnsupportedChannelConfiguration),
			}
		}
		Ok(Self::new(
			reader.ident_hdr.audio_sample_rate,
			stereo_samples,
			settings,
		))
	}

	/// Decodes a sound from a flac file.
	pub fn from_flac_file<P>(path: P, settings: SoundSettings) -> KiraResult<Self>
	where
		P: AsRef<Path>,
	{
		let mut reader = FlacReader::open(path)?;
		let streaminfo = reader.streaminfo();
		let mut stereo_samples = vec![];
		match reader.streaminfo().channels {
			1 => {
				for sample in reader.samples() {
					let sample = sample?;
					stereo_samples.push(StereoSample::from_i32(
						sample,
						sample,
						streaminfo.bits_per_sample,
					));
				}
			}
			2 => {
				let mut iter = reader.samples();
				while let (Some(left), Some(right)) = (iter.next(), iter.next()) {
					stereo_samples.push(StereoSample::from_i32(
						left?,
						right?,
						streaminfo.bits_per_sample,
					));
				}
			}
			_ => return Err(KiraError::UnsupportedChannelConfiguration),
		}
		Ok(Self::new(streaminfo.sample_rate, stereo_samples, settings))
	}

	/// Decodes a sound from a wav file.
	pub fn from_wav_file<P>(path: P, settings: SoundSettings) -> KiraResult<Self>
	where
		P: AsRef<Path>,
	{
		let mut reader = WavReader::open(path)?;
		let spec = reader.spec();
		let mut stereo_samples = vec![];
		match reader.spec().channels {
			1 => match spec.sample_format {
				hound::SampleFormat::Float => {
					for sample in reader.samples::<f32>() {
						stereo_samples.push(StereoSample::from_mono(sample?))
					}
				}
				hound::SampleFormat::Int => {
					for sample in reader.samples::<i32>() {
						let sample = sample?;
						stereo_samples.push(StereoSample::from_i32(
							sample,
							sample,
							spec.bits_per_sample.into(),
						));
					}
				}
			},
			2 => match spec.sample_format {
				hound::SampleFormat::Float => {
					let mut iter = reader.samples::<f32>();
					while let (Some(left), Some(right)) = (iter.next(), iter.next()) {
						stereo_samples.push(StereoSample::new(left?, right?));
					}
				}
				hound::SampleFormat::Int => {
					let mut iter = reader.samples::<i32>();
					while let (Some(left), Some(right)) = (iter.next(), iter.next()) {
						stereo_samples.push(StereoSample::from_i32(
							left?,
							right?,
							spec.bits_per_sample.into(),
						));
					}
				}
			},
			_ => return Err(KiraError::UnsupportedChannelConfiguration),
		}
		Ok(Self::new(
			reader.spec().sample_rate,
			stereo_samples,
			settings,
		))
	}

	/// Decodes a sound from a file.
	///
	/// The audio format will be automatically determined from the file extension.
	pub fn from_file<P>(path: P, settings: SoundSettings) -> KiraResult<Self>
	where
		P: AsRef<Path>,
	{
		if let Some(extension) = path.as_ref().extension() {
			if let Some(extension_str) = extension.to_str() {
				match extension_str {
					"mp3" => return Self::from_mp3_file(path, settings),
					"ogg" => return Self::from_ogg_file(path, settings),
					"flac" => return Self::from_flac_file(path, settings),
					"wav" => return Self::from_wav_file(path, settings),
					_ => {}
				}
			}
		}
		Err(KiraError::UnsupportedAudioFileFormat)
	}

	/// Gets the default track that the sound plays on.
	pub fn default_track(&self) -> TrackIndex {
		self.default_track
	}

	/// Gets the duration of the sound (in seconds).
	pub fn duration(&self) -> f64 {
		self.duration
	}

	/// Gets the metadata associated with the sound.
	pub fn metadata(&self) -> SoundMetadata {
		self.metadata
	}

	/// Gets the sample at an arbitrary time in seconds,
	/// interpolating between samples if necessary.
	pub fn get_sample_at_position(&self, position: f64) -> StereoSample {
		let sample_position = self.sample_rate as f64 * position;
		let x = (sample_position % 1.0) as f32;
		let current_sample_index = sample_position as usize;
		let y0 = if current_sample_index == 0 {
			StereoSample::from_mono(0.0)
		} else {
			*self
				.samples
				.get(current_sample_index - 1)
				.unwrap_or(&StereoSample::from_mono(0.0))
		};
		let y1 = *self
			.samples
			.get(current_sample_index)
			.unwrap_or(&StereoSample::from_mono(0.0));
		let y2 = *self
			.samples
			.get(current_sample_index + 1)
			.unwrap_or(&StereoSample::from_mono(0.0));
		let y3 = *self
			.samples
			.get(current_sample_index + 2)
			.unwrap_or(&StereoSample::from_mono(0.0));
		let c0 = y1;
		let c1 = (y2 - y0) * 0.5;
		let c2 = y0 - y1 * 2.5 + y2 * 2.0 - y3 * 0.5;
		let c3 = (y3 - y0) * 0.5 + (y1 - y2) * 1.5;
		((c3 * x + c2) * x + c1) * x + c0
	}

	pub(crate) fn start_cooldown(&mut self) {
		if let Some(cooldown) = self.cooldown {
			self.cooldown_timer = cooldown;
		}
	}

	pub(crate) fn update_cooldown(&mut self, dt: f64) {
		if self.cooldown_timer > 0.0 {
			self.cooldown_timer -= dt;
		}
	}

	pub(crate) fn cooling_down(&self) -> bool {
		self.cooldown_timer > 0.0
	}
}
