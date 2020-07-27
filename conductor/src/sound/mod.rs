mod instance;

use crate::stereo_sample::StereoSample;
use instance::{Instance, InstanceState};
use lewton::{inside_ogg::OggStreamReader, samples::Samples};
use std::{error::Error, fs::File, path::Path};

const NUM_INSTANCES: usize = 8;

pub struct Sound {
	pub samples: Vec<StereoSample>,
	pub instances: Vec<Instance>,
}

impl Sound {
	pub fn new(samples: Vec<StereoSample>) -> Self {
		let mut instances = vec![];
		for _ in 0..NUM_INSTANCES {
			instances.push(Instance::new(samples.len()));
		}
		Self { samples, instances }
	}

	pub fn from_ogg_file(path: &Path) -> Result<Self, Box<dyn Error>> {
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
		Ok(Self::new(samples))
	}

	fn pick_instance_to_play(&self) -> Option<usize> {
		if let Some((i, _)) = self
			.instances
			.iter()
			.enumerate()
			.find(|(_, instance)| instance.state() == InstanceState::Stopped)
		{
			return Some(i);
		};
		if let Some((i, _)) = self
			.instances
			.iter()
			.enumerate()
			.max_by(|(_, a), (_, b)| a.position().cmp(&b.position()))
		{
			return Some(i);
		}
		None
	}

	pub fn play(&mut self) {
		if let Some(index) = self.pick_instance_to_play() {
			self.instances[index].play();
		}
	}

	pub fn process(&mut self) -> StereoSample {
		let mut out = StereoSample::from_mono(0.0);
		for instance in &mut self.instances {
			if let Some(position) = instance.update() {
				out.left += self.samples[position].left;
				out.right += self.samples[position].right;
			}
		}
		out
	}
}
