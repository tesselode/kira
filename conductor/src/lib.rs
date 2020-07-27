mod stereo_sample;

use cpal::{
	traits::{DeviceTrait, HostTrait, StreamTrait},
	Sample, Stream,
};
use lewton::{inside_ogg::OggStreamReader, samples::Samples};
use std::{collections::HashMap, error::Error, f32::consts::PI, fs::File, hash::Hash, path::Path};
use stereo_sample::StereoSample;

pub struct AudioManager<SoundName>
where
	SoundName: Eq + Hash,
{
	stream: Option<Stream>,
	sounds: HashMap<SoundName, Vec<StereoSample>>,
}

impl<SoundName> AudioManager<SoundName>
where
	SoundName: Eq + Hash,
{
	pub fn new() -> Result<Self, Box<dyn Error>> {
		Ok(Self {
			stream: None,
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

	pub fn start(&mut self) -> Result<(), Box<dyn Error>> {
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
		let mut phase = 0.0;
		let stream = device.build_output_stream(
			&config,
			move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
				for frame in data.chunks_exact_mut(channels as usize) {
					phase += 440.0 / (sample_rate as f32);
					phase %= 1.0;
					let out = 0.25 * (phase * 2.0 * PI).sin();
					for sample in frame.iter_mut() {
						*sample = Sample::from(&out);
					}
				}
			},
			move |err| {},
		)?;
		stream.play()?;
		self.stream = Some(stream);
		Ok(())
	}
}
