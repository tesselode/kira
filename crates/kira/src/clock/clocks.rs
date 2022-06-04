use atomic_arena::{Arena, Controller};
use ringbuf::Producer;

use crate::{
	clock::{Clock, ClockId},
	manager::command::ClockCommand,
};

use super::ClockTime;

pub(crate) struct Clocks {
	clocks: Arena<Clock>,
	unused_clock_producer: Producer<Clock>,
	clock_tick_events: Vec<ClockTime>,
}

impl Clocks {
	pub(crate) fn new(capacity: usize, unused_clock_producer: Producer<Clock>) -> Self {
		Self {
			clocks: Arena::new(capacity),
			unused_clock_producer,
			clock_tick_events: Vec::with_capacity(capacity),
		}
	}

	pub(crate) fn controller(&self) -> Controller {
		self.clocks.controller()
	}

	pub(crate) fn on_start_processing(&mut self) {
		self.remove_unused_clocks();
		for (_, clock) in &mut self.clocks {
			clock.on_start_processing();
		}
	}

	fn remove_unused_clocks(&mut self) {
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

	pub(crate) fn run_command(&mut self, command: ClockCommand) {
		match command {
			ClockCommand::Add(id, clock) => self
				.clocks
				.insert_with_key(id.0, clock)
				.expect("Clock arena is full"),
			ClockCommand::SetSpeed(id, speed, tween) => {
				if let Some(clock) = self.clocks.get_mut(id.0) {
					clock.set_speed(speed, tween);
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

	pub(crate) fn update(&mut self, dt: f64) -> &[ClockTime] {
		self.clock_tick_events.clear();
		for (id, clock) in &mut self.clocks {
			if let Some(ticks) = clock.update(dt) {
				self.clock_tick_events.push(ClockTime {
					clock: ClockId(id),
					ticks,
				});
			}
		}
		for time in &self.clock_tick_events {
			for (_, clock) in &mut self.clocks {
				clock.on_clock_tick(*time);
			}
		}
		&self.clock_tick_events
	}
}
