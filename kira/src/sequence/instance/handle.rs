use ringbuf::Consumer;

pub struct SequenceInstanceHandle<Event> {
	events: Vec<Event>,
	event_consumer: Consumer<usize>,
}

impl<Event> SequenceInstanceHandle<Event> {
	pub(crate) fn new(events: Vec<Event>, event_consumer: Consumer<usize>) -> Self {
		Self {
			events,
			event_consumer,
		}
	}

	pub fn pop_event(&mut self) -> Option<&Event> {
		if let Some(index) = self.event_consumer.pop() {
			Some(&self.events[index])
		} else {
			None
		}
	}
}
