use crate::{stereo_sample::StereoSample, tempo::Tempo};
use claxon::FlacReader;
use lewton::{inside_ogg::OggStreamReader, samples::Samples};
use std::{
	error::Error,
	fs::File,
	hash::Hash,
	path::Path,
	sync::atomic::{AtomicUsize, Ordering},
};

/// Useful info about a `Sound`.
///
/// This is set entirely by the user when loading a sound
/// and can be accessed via `SoundId`s.
#[derive(Debug, Default, Copy, Clone)]
pub struct SoundMetadata {
	pub tempo: Option<Tempo>,
}

static NEXT_SOUND_INDEX: AtomicUsize = AtomicUsize::new(0);

/// A unique identifier for a `Sound`.
///
/// You cannot create this manually - a `SoundId` is returned
/// when you load a sound with a `Project`.
#[derive(Debug, Copy, Clone)]
pub struct SoundId {
	index: usize,
	duration: f64,
	metadata: SoundMetadata,
}

impl SoundId {
	pub fn duration(&self) -> f64 {
		self.duration
	}

	pub fn metadata(&self) -> &SoundMetadata {
		&self.metadata
	}
}

impl PartialEq for SoundId {
	fn eq(&self, other: &Self) -> bool {
		self.index == other.index
	}
}

impl Eq for SoundId {}

impl Hash for SoundId {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.index.hash(state);
	}
}

impl SoundId {
	pub(crate) fn new(duration: f64, metadata: SoundMetadata) -> Self {
		let index = NEXT_SOUND_INDEX.fetch_add(1, Ordering::Relaxed);
		Self {
			index,
			duration,
			metadata,
		}
	}
}

#[derive(Debug)]
pub struct SoundSettings {
	pub cooldown: Option<f64>,
	pub metadata: SoundMetadata,
}

impl Default for SoundSettings {
	fn default() -> Self {
		Self {
			cooldown: Some(0.0001),
			metadata: SoundMetadata::default(),
		}
	}
}

#[derive(Debug)]
pub(crate) struct Sound {
	sample_rate: u32,
	samples: Vec<StereoSample>,
	duration: f64,
	cooldown: Option<f64>,
	cooldown_timer: f64,
}

impl Sound {
	pub fn new(sample_rate: u32, samples: Vec<StereoSample>, settings: &SoundSettings) -> Self {
		let duration = samples.len() as f64 / sample_rate as f64;
		Self {
			sample_rate,
			samples,
			duration,
			cooldown: settings.cooldown,
			cooldown_timer: 0.0,
		}
	}

	pub fn from_ogg_file<P>(path: P, settings: &SoundSettings) -> Result<Self, Box<dyn Error>>
	where
		P: AsRef<Path>,
	{
		let mut reader = OggStreamReader::new(File::open(path)?)?;
		let mut samples = vec![];
		while let Some(packet) = reader.read_dec_packet_generic::<Vec<Vec<f32>>>()? {
			let num_channels = packet.len();
			let num_samples = packet.num_samples();
			match num_channels {
				1 => {
					for i in 0..num_samples {
						samples.push(StereoSample::from_mono(packet[0][i]));
					}
				}
				2 => {
					for i in 0..num_samples {
						samples.push(StereoSample::new(packet[0][i], packet[1][i]));
					}
				}
				_ => {
					panic!("Only mono and stereo audio can be loaded");
				}
			}
		}
		Ok(Self::new(
			reader.ident_hdr.audio_sample_rate,
			samples,
			settings,
		))
	}

	pub fn from_flac_file<P>(path: P, settings: &SoundSettings) -> Result<Self, Box<dyn Error>>
	where
		P: AsRef<Path>,
	{
		let mut reader = FlacReader::open(path)?;
		let mut samples = vec![];
		let range = (1 << reader.streaminfo().bits_per_sample) / 2;
		match reader.streaminfo().channels {
			1 => {
				for sample in reader.samples() {
					let sample = sample?;
					samples.push(StereoSample::from_mono(sample as f32 / range as f32));
				}
			}
			2 => {
				let mut stereo_sample = StereoSample::new(0.0, 0.0);
				let mut reading_right_channel = false;
				for sample in reader.samples() {
					let sample = sample?;
					if reading_right_channel {
						stereo_sample.right = sample as f32 / range as f32;
						samples.push(stereo_sample);
						stereo_sample = StereoSample::new(0.0, 0.0);
						reading_right_channel = false;
					} else {
						stereo_sample.left = sample as f32 / range as f32;
						reading_right_channel = true;
					}
				}
			}
			_ => {
				panic!("Only mono and stereo audio can be loaded");
			}
		}
		Ok(Self::new(
			reader.streaminfo().sample_rate,
			samples,
			settings,
		))
	}

	pub fn from_file<P>(path: P, settings: &SoundSettings) -> Result<Self, Box<dyn Error>>
	where
		P: AsRef<Path>,
	{
		if let Some(extension) = path.as_ref().extension() {
			if let Some(extension_str) = extension.to_str() {
				match extension_str {
					"ogg" => return Self::from_ogg_file(path, settings),
					"flac" => return Self::from_flac_file(path, settings),
					_ => {}
				}
			}
		}
		panic!("Unsupported file format");
	}

	pub fn duration(&self) -> f64 {
		self.duration
	}

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
