use atomic_arena::{Arena, Controller};
use ringbuf::Producer;

use crate::{
	clock::{Clock, ClockId},
	manager::command::ClockCommand,
};

use super::Parameters;

pub(crate) struct Clocks {
	clocks: Arena<Clock>,
	unused_clock_producer: Producer<Clock>,
}

impl Clocks {
	pub fn new(capacity: usize, unused_clock_producer: Producer<Clock>) -> Self {
		Self {
			clocks: Arena::new(capacity),
			unused_clock_producer,
		}
	}

	pub fn controller(&self) -> Controller {
		self.clocks.controller()
	}

	pub fn get(&self, id: ClockId) -> Option<&Clock> {
		self.clocks.get(id.0)
	}

	pub fn on_start_processing(&mut self) {
		if self.unused_clock_producer.is_full() {
			return;
		}
		for (_, clock) in self
			.clocks
			.drain_filter(|clock| clock.shared().is_marked_for_removal())
		{
			if self.unused_clock_producer.push(clock).is_err() {
				panic!("Unused clock producer is full")
			}
			if self.unused_clock_producer.is_full() {
				return;
			}
		}
	}

	pub fn run_command(&mut self, command: ClockCommand) {
		match command {
			ClockCommand::Add(id, clock) => self
				.clocks
				.insert_with_index(id.0, clock)
				.expect("Clock arena is full"),
			ClockCommand::SetInterval(id, interval) => {
				if let Some(clock) = self.clocks.get_mut(id.0) {
					clock.set_interval(interval);
				}
			}
			ClockCommand::Start(id) => {
				if let Some(clock) = self.clocks.get_mut(id.0) {
					clock.start();
				}
			}
			ClockCommand::Pause(id) => {
				if let Some(clock) = self.clocks.get_mut(id.0) {
					clock.pause();
				}
			}
			ClockCommand::Stop(id) => {
				if let Some(clock) = self.clocks.get_mut(id.0) {
					clock.stop();
				}
			}
		}
	}

	pub fn update(&mut self, dt: f64, parameters: &Parameters) {
		for (_, clock) in &mut self.clocks {
			clock.update(dt, parameters);
		}
	}
}
