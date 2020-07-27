mod backend;
mod stereo_sample;

use backend::Backend;
use cpal::{
	traits::{DeviceTrait, HostTrait, StreamTrait},
	Sample, Stream,
};
use lewton::{inside_ogg::OggStreamReader, samples::Samples};
use std::{collections::HashMap, error::Error, fs::File, hash::Hash, path::Path};
use stereo_sample::StereoSample;

pub struct AudioManager<SoundName>
where
	SoundName: Eq + Hash,
{
	stream: Stream,
	sounds: HashMap<SoundName, Vec<StereoSample>>,
}

impl<SoundName> AudioManager<SoundName>
where
	SoundName: Eq + Hash,
{
	pub fn new() -> Result<Self, Box<dyn Error>> {
		let host = cpal::default_host();
		let device = host.default_output_device().unwrap();
		let mut supported_configs_range = device.supported_output_configs().unwrap();
		let supported_config = supported_configs_range
			.next()
			.unwrap()
			.with_max_sample_rate();
		let config = supported_config.config();
		let sample_rate = config.sample_rate.0;
		let channels = config.channels;
		let mut backend = Backend::new(sample_rate);
		let stream = device.build_output_stream(
			&config,
			move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
				for frame in data.chunks_exact_mut(channels as usize) {
					let out = backend.process();
					frame[0] = out.left;
					frame[1] = out.right;
				}
			},
			move |err| {},
		)?;
		stream.play()?;
		Ok(Self {
			stream: stream,
			sounds: HashMap::new(),
		})
	}

	pub fn load_sound(&mut self, sound_name: SoundName, path: &Path) -> Result<(), Box<dyn Error>> {
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
		self.sounds.insert(sound_name, samples);
		Ok(())
	}
}
