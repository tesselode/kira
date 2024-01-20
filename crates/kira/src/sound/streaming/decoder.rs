#[cfg(test)]
pub(crate) mod mock;
#[cfg(feature = "symphonia")]
pub(crate) mod symphonia;

use crate::dsp::Frame;

/// Decodes chunks of audio.
pub trait Decoder: Send {
	/// Errors that can occur when decoding audio.
	type Error;

	/// Returns the sample rate of the audio (in Hz).
	fn sample_rate(&self) -> u32;

	/// Returns the total number of samples of audio.
	fn num_frames(&self) -> usize;

	/// Decodes the next chunk of audio.
	fn decode(&mut self) -> Result<Vec<Frame>, Self::Error>;

	/// Seeks to an audio sample.
	///
	/// The `index` is the _requested_ sample to seek to. It's OK if the decoder
	/// seeks to an earlier sample than the one requested.
	///
	/// This should return the sample index that was _actually_ seeked to.
	fn seek(&mut self, index: usize) -> Result<SeekedToIndex, Self::Error>;
}

type SeekedToIndex = usize;
