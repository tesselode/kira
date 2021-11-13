use std::{
	fmt::Display,
	fs::File,
	io::{Read, Seek},
	path::Path,
	sync::Arc,
};

use kira::sound::static_sound::{Samples, StaticSoundData, StaticSoundSettings};

#[derive(Debug)]
pub enum DecodeError {
	UnsupportedChannelConfiguration,
	VariableSampleRate,
	Mp3Error(minimp3::Error),
}

impl Display for DecodeError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			DecodeError::UnsupportedChannelConfiguration => {
				f.write_str("Only mono and stereo audio is supported")
			}
			DecodeError::VariableSampleRate => {
				f.write_str("The audio has a variable sample rate, which is not supported")
			}
			DecodeError::Mp3Error(error) => error.fmt(f),
		}
	}
}

impl std::error::Error for DecodeError {
	fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
		match self {
			DecodeError::Mp3Error(error) => Some(error),
			_ => None,
		}
	}
}

impl From<minimp3::Error> for DecodeError {
	fn from(v: minimp3::Error) -> Self {
		Self::Mp3Error(v)
	}
}

#[derive(Debug)]
pub enum FromFileError {
	IoError(std::io::Error),
	DecodeError(DecodeError),
}

impl Display for FromFileError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			FromFileError::IoError(error) => error.fmt(f),
			FromFileError::DecodeError(error) => error.fmt(f),
		}
	}
}

impl std::error::Error for FromFileError {
	fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
		match self {
			FromFileError::IoError(error) => Some(error),
			FromFileError::DecodeError(error) => Some(error),
		}
	}
}

impl From<std::io::Error> for FromFileError {
	fn from(v: std::io::Error) -> Self {
		Self::IoError(v)
	}
}

impl From<DecodeError> for FromFileError {
	fn from(v: DecodeError) -> Self {
		Self::DecodeError(v)
	}
}

pub fn from_reader(
	reader: impl Read + Seek,
	settings: StaticSoundSettings,
) -> Result<StaticSoundData, DecodeError> {
	let mut decoder = minimp3::Decoder::new(reader);
	let mut sample_rate = None;
	// start off assuming the sound is mono. if we encounter
	// a stereo sample, we'll convert the mono samples to
	// stereo and push stereo samples from then on.
	//
	// in practice, i don't know if there's a single mp3 file
	// in the world that actually has different channel counts
	// for different frames, but the minimp3 API implies that
	// this is possible, since each Frame has its own channels
	// field.
	let mut samples = Samples::I16Mono(vec![]);
	loop {
		match decoder.next_frame() {
			Ok(mut frame) => {
				if let Some(sample_rate) = sample_rate {
					if frame.sample_rate as u32 != sample_rate {
						return Err(DecodeError::VariableSampleRate);
					}
				} else {
					sample_rate = Some(frame.sample_rate as u32);
				}
				match frame.channels {
					1 => match &mut samples {
						Samples::I16Mono(samples) => {
							samples.append(&mut frame.data);
						}
						Samples::I16Stereo(samples) => {
							for sample in frame.data {
								samples.push([sample, sample]);
							}
						}
						_ => unreachable!(),
					},
					2 => {
						samples = if let Samples::I16Mono(samples) = samples {
							Samples::I16Stereo(convert_i16_mono_to_stereo(samples))
						} else {
							samples
						};
						match &mut samples {
							Samples::I16Stereo(samples) => {
								for chunk in frame.data.chunks_exact(2) {
									samples.push([chunk[0], chunk[1]]);
								}
							}
							_ => unreachable!(),
						}
					}
					_ => return Err(DecodeError::UnsupportedChannelConfiguration),
				}
			}
			Err(error) => match error {
				minimp3::Error::Eof => break,
				error => return Err(error.into()),
			},
		}
	}
	Ok(StaticSoundData {
		sample_rate: sample_rate.unwrap_or(1),
		samples: Arc::new(samples),
		settings,
	})
}

pub fn from_file(
	path: impl AsRef<Path>,
	settings: StaticSoundSettings,
) -> Result<StaticSoundData, FromFileError> {
	Ok(from_reader(File::open(path)?, settings)?)
}

fn convert_i16_mono_to_stereo(samples: Vec<i16>) -> Vec<[i16; 2]> {
	samples.iter().map(|sample| [*sample, *sample]).collect()
}
