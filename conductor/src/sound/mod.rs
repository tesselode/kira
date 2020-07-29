mod instance;

use crate::{manager::PlaySoundSettings, stereo_sample::StereoSample};
use instance::{Instance, InstanceState};
use lewton::{inside_ogg::OggStreamReader, samples::Samples};
use std::{error::Error, fs::File, path::Path};

const NUM_INSTANCES: usize = 8;

pub struct Sound {
	sample_rate: u32,
	pub samples: Vec<StereoSample>,
	pub instances: Vec<Instance>,
}

fn get_interpolated_sample(
	samples: &Vec<StereoSample>,
	sample_rate: u32,
	position: f32,
) -> StereoSample {
	let sample_position = sample_rate as f32 * position;
	let x = sample_position % 1.0;
	let current_sample_index = sample_position as usize;
	let y0 = if current_sample_index == 0 {
		StereoSample::from_mono(0.0)
	} else {
		*samples
			.get(current_sample_index - 1)
			.unwrap_or(&StereoSample::from_mono(0.0))
	};
	let y1 = *samples
		.get(current_sample_index)
		.unwrap_or(&StereoSample::from_mono(0.0));
	let y2 = *samples
		.get(current_sample_index + 1)
		.unwrap_or(&StereoSample::from_mono(0.0));
	let y3 = *samples
		.get(current_sample_index + 2)
		.unwrap_or(&StereoSample::from_mono(0.0));
	let c0 = y1;
	let c1 = (y2 - y0) * 0.5;
	let c2 = y0 - y1 * 2.5 + y2 * 2.0 - y3 * 0.5;
	let c3 = (y3 - y0) * 0.5 + (y1 - y2) * 1.5;
	((c3 * x + c2) * x + c1) * x + c0
}

impl Sound {
	pub fn new(sample_rate: u32, samples: Vec<StereoSample>) -> Self {
		let duration = samples.len() as f32 / sample_rate as f32;
		let mut instances = vec![];
		for _ in 0..NUM_INSTANCES {
			instances.push(Instance::new(duration));
		}
		Self {
			sample_rate,
			samples,
			instances,
		}
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
		Ok(Self::new(reader.ident_hdr.audio_sample_rate, samples))
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
			.max_by(|(_, a), (_, b)| a.position().partial_cmp(&b.position()).unwrap())
		{
			return Some(i);
		}
		None
	}

	pub fn play(&mut self, settings: PlaySoundSettings) {
		if let Some(index) = self.pick_instance_to_play() {
			self.instances[index].play(settings);
		}
	}

	pub fn process(&mut self, dt: f32) -> StereoSample {
		let mut out = StereoSample::from_mono(0.0);
		for instance in &mut self.instances {
			if let Some(position) = instance.update(dt) {
				out += get_interpolated_sample(&self.samples, self.sample_rate, position)
					* instance.volume();
			}
		}
		out
	}
}
