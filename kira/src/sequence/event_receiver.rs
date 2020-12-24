use indexmap::IndexSet;
use ringbuf::Consumer;

/// Receives events from an instance of a [`Sequence`](crate::sequence::Sequence).
pub struct EventReceiver<CustomEvent> {
	consumer: Consumer<usize>,
	events: IndexSet<CustomEvent>,
}

impl<CustomEvent> EventReceiver<CustomEvent> {
	pub(crate) fn new(consumer: Consumer<usize>, events: IndexSet<CustomEvent>) -> Self {
		Self { consumer, events }
	}

	/// Gets the first event that was emitted since the last
	/// call to `pop`.
	pub fn pop(&mut self) -> Option<&CustomEvent> {
		self.consumer
			.pop()
			.map(move |index| self.events.get_index(index).unwrap())
	}
}
