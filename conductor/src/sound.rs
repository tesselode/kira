use crate::stereo_sample::StereoSample;
use lewton::{inside_ogg::OggStreamReader, samples::Samples};
use std::{error::Error, fs::File, path::Path};

pub struct Sound {
	pub(crate) samples: Vec<StereoSample>,
}

impl Sound {
	pub(crate) fn new(samples: Vec<StereoSample>) -> Self {
		Self { samples }
	}

	pub(crate) fn from_ogg_file(path: &Path) -> Result<Self, Box<dyn Error>> {
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
		Ok(Self { samples })
	}
}
