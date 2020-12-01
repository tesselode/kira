//! A low level abstraction over any audio stream.

use std::fmt::Debug;

use crate::Frame;

/// The AudioStream trait describes a source of [Frame](Frame).
/// AudioStream structs are run on the audio thread, and must therefore
/// provide samples with very low latency in order to avoid introducing delay and
/// audio artifacts. It is recommanded you only use an AudioStream if you
/// know you need it as they offer *virtually no features*.
/// This is only useful if you need to have a custom audio
/// stream within the Kira context without satisfying
/// all the constraints that enable some of the features.
/// An AudioStream can be set as the background stream of a track
/// via the [AudioManager](crate::manager::AudioManager).
pub trait AudioStream: Debug + Send + 'static {
    /// Called every time the mixer requires a sample to be played immediately.
    /// It is crucial this method returns shortly as it is part of the audio thread's
    /// main loop.
    /// `dt` represents how many seconds have elapsed since the last request.
    fn next(&mut self, dt: f64) -> Frame;
}
