pub mod error;

use crate::{frame::Frame, util};

#[cfg(any(feature = "mp3", feature = "ogg", feature = "flac", feature = "wav"))]
use std::{
	fs::File,
	io::{Read, Seek},
	path::Path,
};

use super::SoundData;

pub struct StaticSoundData {
	sample_rate: u32,
	duration: f64,
	frames: Vec<Frame>,
}

impl StaticSoundData {
	/// Creates a new sound from raw sample data.
	pub fn from_frames(sample_rate: u32, frames: Vec<Frame>) -> Self {
		let duration = frames.len() as f64 / sample_rate as f64;
		Self {
			sample_rate,
			frames,
			duration,
		}
	}

	/// Decodes a sound from an mp3 reader.
	#[cfg(feature = "mp3")]
	pub fn from_mp3_reader<R>(reader: R) -> Result<Self, error::SoundFromFileError>
	where
		R: Read,
	{
		let mut decoder = minimp3::Decoder::new(reader);
		let mut sample_rate = None;
		let mut stereo_samples = vec![];
		loop {
			match decoder.next_frame() {
				Ok(frame) => {
					if let Some(sample_rate) = sample_rate {
						if sample_rate != frame.sample_rate {
							return Err(error::SoundFromFileError::VariableMp3SampleRate);
						}
					} else {
						sample_rate = Some(frame.sample_rate);
					}
					match frame.channels {
						1 => {
							for sample in frame.data {
								stereo_samples.push(Frame::from_i32(
									sample.into(),
									sample.into(),
									16,
								))
							}
						}
						2 => {
							let mut iter = frame.data.iter();
							while let (Some(left), Some(right)) = (iter.next(), iter.next()) {
								stereo_samples.push(Frame::from_i32(
									(*left).into(),
									(*right).into(),
									16,
								))
							}
						}
						_ => {
							return Err(error::SoundFromFileError::UnsupportedChannelConfiguration)
						}
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
			None => return Err(error::SoundFromFileError::UnknownMp3SampleRate),
		};
		Ok(Self::from_frames(sample_rate as u32, stereo_samples))
	}

	/// Decodes a sound from an mp3 file.
	#[cfg(feature = "mp3")]
	pub fn from_mp3_file<P>(path: P) -> Result<Self, error::SoundFromFileError>
	where
		P: AsRef<Path>,
	{
		Self::from_mp3_reader(File::open(path)?)
	}

	/// Decodes a sound from an ogg reader.
	#[cfg(feature = "ogg")]
	pub fn from_ogg_reader<R>(reader: R) -> Result<Self, error::SoundFromFileError>
	where
		R: Read + Seek,
	{
		use lewton::{inside_ogg::OggStreamReader, samples::Samples};
		let mut reader = OggStreamReader::new(reader)?;
		let mut stereo_samples = vec![];
		while let Some(packet) = reader.read_dec_packet_generic::<Vec<Vec<f32>>>()? {
			let num_channels = packet.len();
			let num_samples = packet.num_samples();
			match num_channels {
				1 => {
					for i in 0..num_samples {
						stereo_samples.push(Frame::from_mono(packet[0][i]));
					}
				}
				2 => {
					for i in 0..num_samples {
						stereo_samples.push(Frame::new(packet[0][i], packet[1][i]));
					}
				}
				_ => return Err(error::SoundFromFileError::UnsupportedChannelConfiguration),
			}
		}
		Ok(Self::from_frames(
			reader.ident_hdr.audio_sample_rate,
			stereo_samples,
		))
	}

	/// Decodes a sound from an ogg file.
	#[cfg(feature = "ogg")]
	pub fn from_ogg_file<P>(path: P) -> Result<Self, error::SoundFromFileError>
	where
		P: AsRef<Path>,
	{
		Self::from_ogg_reader(File::open(path)?)
	}

	/// Decodes a sound from a flac file.
	#[cfg(feature = "flac")]
	pub fn from_flac_reader<R>(reader: R) -> Result<Self, error::SoundFromFileError>
	where
		R: Read,
	{
		let mut reader = claxon::FlacReader::new(reader)?;
		let streaminfo = reader.streaminfo();
		let mut stereo_samples = vec![];
		match reader.streaminfo().channels {
			1 => {
				for sample in reader.samples() {
					let sample = sample?;
					stereo_samples.push(Frame::from_i32(
						sample,
						sample,
						streaminfo.bits_per_sample,
					));
				}
			}
			2 => {
				let mut iter = reader.samples();
				while let (Some(left), Some(right)) = (iter.next(), iter.next()) {
					stereo_samples.push(Frame::from_i32(left?, right?, streaminfo.bits_per_sample));
				}
			}
			_ => return Err(error::SoundFromFileError::UnsupportedChannelConfiguration),
		}
		Ok(Self::from_frames(streaminfo.sample_rate, stereo_samples))
	}

	/// Decodes sound from a flac reader.
	#[cfg(feature = "flac")]
	pub fn from_flac_file<P>(path: P) -> Result<Self, error::SoundFromFileError>
	where
		P: AsRef<Path>,
	{
		Self::from_flac_reader(File::open(path)?)
	}

	/// Decodes sound from a wav reader.
	#[cfg(feature = "wav")]
	pub fn from_wav_reader<R>(reader: R) -> Result<Self, error::SoundFromFileError>
	where
		R: Read,
	{
		let mut reader = hound::WavReader::new(reader)?;
		let spec = reader.spec();
		let mut stereo_samples = vec![];
		match reader.spec().channels {
			1 => match spec.sample_format {
				hound::SampleFormat::Float => {
					for sample in reader.samples::<f32>() {
						stereo_samples.push(Frame::from_mono(sample?))
					}
				}
				hound::SampleFormat::Int => {
					for sample in reader.samples::<i32>() {
						let sample = sample?;
						stereo_samples.push(Frame::from_i32(
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
						stereo_samples.push(Frame::new(left?, right?));
					}
				}
				hound::SampleFormat::Int => {
					let mut iter = reader.samples::<i32>();
					while let (Some(left), Some(right)) = (iter.next(), iter.next()) {
						stereo_samples.push(Frame::from_i32(
							left?,
							right?,
							spec.bits_per_sample.into(),
						));
					}
				}
			},
			_ => return Err(error::SoundFromFileError::UnsupportedChannelConfiguration),
		}
		Ok(Self::from_frames(reader.spec().sample_rate, stereo_samples))
	}

	/// Decodes a sound from a wav file.
	#[cfg(feature = "wav")]
	pub fn from_wav_file<P>(path: P) -> Result<Self, error::SoundFromFileError>
	where
		P: AsRef<Path>,
	{
		Self::from_wav_reader(File::open(path)?)
	}

	/// Decodes a sound from a file.
	///
	/// The audio format will be automatically determined from the file extension.
	#[cfg(any(feature = "mp3", feature = "ogg", feature = "flac", feature = "wav"))]
	pub fn from_file<P>(path: P) -> Result<Self, error::SoundFromFileError>
	where
		P: AsRef<Path>,
	{
		if let Some(extension) = path.as_ref().extension() {
			if let Some(extension_str) = extension.to_str() {
				match extension_str {
					#[cfg(feature = "mp3")]
					"mp3" => return Self::from_mp3_file(path),
					#[cfg(feature = "ogg")]
					"ogg" => return Self::from_ogg_file(path),
					#[cfg(feature = "flac")]
					"flac" => return Self::from_flac_file(path),
					#[cfg(feature = "wav")]
					"wav" => return Self::from_wav_file(path),
					_ => {}
				}
			}
		}
		Err(error::SoundFromFileError::UnsupportedAudioFileFormat)
	}
}

impl SoundData for StaticSoundData {
	fn duration(&self) -> f64 {
		self.duration
	}

	fn frame_at_position(&self, position: f64) -> Frame {
		let sample_position = self.sample_rate as f64 * position;
		let fraction = (sample_position % 1.0) as f32;
		let current_sample_index = sample_position as usize;
		let previous = if current_sample_index == 0 {
			Frame::from_mono(0.0)
		} else {
			*self
				.frames
				.get(current_sample_index - 1)
				.unwrap_or(&Frame::from_mono(0.0))
		};
		let current = *self
			.frames
			.get(current_sample_index)
			.unwrap_or(&Frame::from_mono(0.0));
		let next_1 = *self
			.frames
			.get(current_sample_index + 1)
			.unwrap_or(&Frame::from_mono(0.0));
		let next_2 = *self
			.frames
			.get(current_sample_index + 2)
			.unwrap_or(&Frame::from_mono(0.0));
		util::interpolate_frame(previous, current, next_1, next_2, fraction)
	}
}
