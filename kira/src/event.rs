/// An audio-related event that can be observed on the main thread.
#[derive(Debug, Copy, Clone)]
pub enum Event {
	/**
	Sent when the metronome passes a certain interval (in beats).

	For example, an event with an interval of `1.0` will be sent
	every beat, and an event with an interval of `0.25` will be
	sent every sixteenth note (one quarter of a beat).

	The intervals that a metronome emits events for are defined
	when the metronome is created.
	*/
	MetronomeIntervalPassed(f64),
}
