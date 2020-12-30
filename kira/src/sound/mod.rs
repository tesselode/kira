//! Provides an interface to work with pieces of audio.

mod handle;
mod id;

pub use handle::SoundHandle;
pub use id::SoundId;

use indexmap::IndexSet;

use crate::{
	frame::Frame,
	group::{groups::Groups, GroupId},
	mixer::TrackIndex,
	playable::PlayableSettings,
	util::index_set_from_vec,
};

#[cfg(any(feature = "mp3", feature = "ogg", feature = "flac", feature = "wav"))]
use crate::{error::AudioError, error::AudioResult};

use std::fmt::{Debug, Formatter};

#[cfg(any(feature = "mp3", feature = "ogg", feature = "flac", feature = "wav"))]
use std::{
	fs::File,
	path::Path,
};

/// A piece of audio that can be played by an [`AudioManager`](crate::manager::AudioManager).
#[derive(Clone)]
pub struct Sound {
	sample_rate: u32,
	frames: Vec<Frame>,
	duration: f64,
	default_track: TrackIndex,
	cooldown: Option<f64>,
	semantic_duration: Option<f64>,
	default_loop_start: Option<f64>,
	groups: IndexSet<GroupId>,
	cooldown_timer: f64,
}

impl Sound {
	/// Creates a new sound from raw sample data.
	pub fn from_frames(sample_rate: u32, frames: Vec<Frame>, settings: PlayableSettings) -> Self {
		let duration = frames.len() as f64 / sample_rate as f64;
		Self {
			sample_rate,
			frames,
			duration,
			default_track: settings.default_track,
			cooldown: settings.cooldown,
			semantic_duration: settings.semantic_duration,
			default_loop_start: settings.default_loop_start,
			groups: index_set_from_vec(settings.groups),
			cooldown_timer: 0.0,
		}
	}

	/// Decodes a sound from an mp3 file.
	#[cfg(feature = "mp3")]
	pub fn from_mp3_file<P>(path: P, settings: PlayableSettings) -> AudioResult<Self>
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
							return Err(AudioError::VariableMp3SampleRate);
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
						_ => return Err(AudioError::UnsupportedChannelConfiguration),
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
			None => return Err(AudioError::UnknownMp3SampleRate),
		};
		Ok(Self::from_frames(
			sample_rate as u32,
			stereo_samples,
			settings,
		))
	}

	/// Decodes a sound from an ogg file.
	#[cfg(feature = "ogg")]
	pub fn from_ogg_file<P>(path: P, settings: PlayableSettings) -> AudioResult<Self>
	where
		P: AsRef<Path>,
	{
		use lewton::{inside_ogg::OggStreamReader, samples::Samples};
		let mut reader = OggStreamReader::new(File::open(path)?)?;
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
				_ => return Err(AudioError::UnsupportedChannelConfiguration),
			}
		}
		Ok(Self::from_frames(
			reader.ident_hdr.audio_sample_rate,
			stereo_samples,
			settings,
		))
	}

	/// Decodes a sound from a flac file.
	#[cfg(feature = "flac")]
	pub fn from_flac_file<P>(path: P, settings: PlayableSettings) -> AudioResult<Self>
	where
		P: AsRef<Path>,
	{
		let mut reader = claxon::FlacReader::open(path)?;
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
			_ => return Err(AudioError::UnsupportedChannelConfiguration),
		}
		Ok(Self::from_frames(
			streaminfo.sample_rate,
			stereo_samples,
			settings,
		))
	}

	/// Decodes a sound from a wav file.
	#[cfg(feature = "wav")]
	pub fn from_wav_file<P>(path: P, settings: PlayableSettings) -> AudioResult<Self>
	where
		P: AsRef<Path>,
	{
		let mut reader = hound::WavReader::open(path)?;
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
			_ => return Err(AudioError::UnsupportedChannelConfiguration),
		}
		Ok(Self::from_frames(
			reader.spec().sample_rate,
			stereo_samples,
			settings,
		))
	}

	/// Decodes a sound from a file.
	///
	/// The audio format will be automatically determined from the file extension.
	#[cfg(any(feature = "mp3", feature = "ogg", feature = "flac", feature = "wav"))]
	pub fn from_file<P>(path: P, settings: PlayableSettings) -> AudioResult<Self>
	where
		P: AsRef<Path>,
	{
		if let Some(extension) = path.as_ref().extension() {
			if let Some(extension_str) = extension.to_str() {
				match extension_str {
					#[cfg(feature = "mp3")]
					"mp3" => return Self::from_mp3_file(path, settings),
					#[cfg(feature = "ogg")]
					"ogg" => return Self::from_ogg_file(path, settings),
					#[cfg(feature = "flac")]
					"flac" => return Self::from_flac_file(path, settings),
					#[cfg(feature = "wav")]
					"wav" => return Self::from_wav_file(path, settings),
					_ => {}
				}
			}
		}
		Err(AudioError::UnsupportedAudioFileFormat)
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
	pub fn semantic_duration(&self) -> Option<f64> {
		self.semantic_duration
	}

	/// Gets the default loop start point for instances
	/// of this sound.
	pub fn default_loop_start(&self) -> Option<f64> {
		self.default_loop_start
	}

	/// Gets the frame of this sound at an arbitrary time
	/// in seconds, interpolating between samples if necessary.
	pub fn get_frame_at_position(&self, position: f64) -> Frame {
		let sample_position = self.sample_rate as f64 * position;
		let x = (sample_position % 1.0) as f32;
		let current_sample_index = sample_position as usize;
		let y0 = if current_sample_index == 0 {
			Frame::from_mono(0.0)
		} else {
			*self
				.frames
				.get(current_sample_index - 1)
				.unwrap_or(&Frame::from_mono(0.0))
		};
		let y1 = *self
			.frames
			.get(current_sample_index)
			.unwrap_or(&Frame::from_mono(0.0));
		let y2 = *self
			.frames
			.get(current_sample_index + 1)
			.unwrap_or(&Frame::from_mono(0.0));
		let y3 = *self
			.frames
			.get(current_sample_index + 2)
			.unwrap_or(&Frame::from_mono(0.0));
		let c0 = y1;
		let c1 = (y2 - y0) * 0.5;
		let c2 = y0 - y1 * 2.5 + y2 * 2.0 - y3 * 0.5;
		let c3 = (y3 - y0) * 0.5 + (y1 - y2) * 1.5;
		((c3 * x + c2) * x + c1) * x + c0
	}

	/// Starts the cooldown timer for the sound.
	pub(crate) fn start_cooldown(&mut self) {
		if let Some(cooldown) = self.cooldown {
			self.cooldown_timer = cooldown;
		}
	}

	/// Updates the cooldown timer for the sound.
	pub(crate) fn update_cooldown(&mut self, dt: f64) {
		if self.cooldown_timer > 0.0 {
			self.cooldown_timer -= dt;
		}
	}

	/// Gets whether the sound is currently "cooling down".
	///
	/// If it is, a new instance of the sound should not
	/// be started until the timer is up.
	pub(crate) fn cooling_down(&self) -> bool {
		self.cooldown_timer > 0.0
	}

	/// Returns if this sound is in the group with the given ID.
	pub(crate) fn is_in_group(&self, parent_id: GroupId, groups: &Groups) -> bool {
		if groups.get(parent_id).is_none() {
			return false;
		}
		// check if this sound is a direct descendant of the requested group
		if self.groups.contains(&parent_id) {
			return true;
		}
		// otherwise, recursively check if any of the direct parents of this
		// sound is in the requested group
		for id in &self.groups {
			if let Some(group) = groups.get(*id) {
				if group.is_in_group(parent_id, groups) {
					return true;
				}
			}
		}
		false
	}
}

impl Debug for Sound {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		f.debug_struct(&format!("Sound ({} frames)", self.frames.len()))
			.field("sample_rate", &self.sample_rate)
			.field("duration", &self.duration)
			.field("default_track", &self.default_track)
			.field("cooldown", &self.cooldown)
			.field("semantic_duration", &self.semantic_duration)
			.field("default_loop_start", &self.default_loop_start)
			.field("groups", &self.groups)
			.field("cooldown_timer", &self.cooldown_timer)
			.finish()
	}
}
