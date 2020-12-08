use indexmap::IndexSet;
use ringbuf::Consumer;

pub struct EventReceiver<CustomEvent> {
	consumer: Consumer<usize>,
	events: IndexSet<CustomEvent>,
}

impl<CustomEvent> EventReceiver<CustomEvent> {
	pub(crate) fn new(consumer: Consumer<usize>, events: IndexSet<CustomEvent>) -> Self {
		Self { consumer, events }
	}

	pub fn pop_event(&mut self) -> Option<&CustomEvent> {
		self.consumer
			.pop()
			.map(move |index| self.events.get_index(index).unwrap())
	}
}
