//! An interface for controlling metronomes.

use std::sync::{Arc, Mutex};

use ringbuf::Consumer;
use thiserror::Error;

use crate::{
	command::{
		producer::{CommandError, CommandProducer},
		MetronomeCommand,
	},
	Tempo, Value,
};

use super::MetronomeId;

/// Something that can go wrong when using a [`MetronomeHandle`]
/// to receive an event from a metronome.
#[derive(Debug, Error)]
pub enum PopMetronomeEventError {
	/// A thread panicked while using the event consumer.
	#[error("The event consumer cannot be used because a thread panicked while borrowing it.")]
	MutexPoisoned,
}

#[derive(Clone)]
/// Allows you to control a metronome.

pub struct MetronomeHandle {
	id: MetronomeId,
	command_producer: CommandProducer,
	event_consumer: Arc<Mutex<Consumer<f64>>>,
}

impl MetronomeHandle {
	pub(crate) fn new(
		id: MetronomeId,
		command_producer: CommandProducer,
		event_consumer: Consumer<f64>,
	) -> Self {
		Self {
			id,
			command_producer,
			event_consumer: Arc::new(Mutex::new(event_consumer)),
		}
	}

	/// Gets the ID of the metronome.
	pub fn id(&self) -> MetronomeId {
		self.id
	}

	/// Sets the tempo of the metronome (in beats per minute).
	pub fn set_tempo(&mut self, tempo: impl Into<Value<Tempo>>) -> Result<(), CommandError> {
		self.command_producer
			.push(MetronomeCommand::SetMetronomeTempo(self.id(), tempo.into()).into())
	}

	/// Starts the metronome.
	pub fn start(&mut self) -> Result<(), CommandError> {
		self.command_producer
			.push(MetronomeCommand::StartMetronome(self.id()).into())
	}

	/// Pauses the metronome.
	pub fn pause(&mut self) -> Result<(), CommandError> {
		self.command_producer
			.push(MetronomeCommand::PauseMetronome(self.id()).into())
	}

	/// Stops the metronome and resets its time to zero.
	pub fn stop(&mut self) -> Result<(), CommandError> {
		self.command_producer
			.push(MetronomeCommand::StopMetronome(self.id()).into())
	}

	/// Gets the first interval event that was emitted by this
	/// metronome since the last call to `pop_event`.
	pub fn pop_event(&mut self) -> Result<Option<f64>, PopMetronomeEventError> {
		Ok(self
			.event_consumer
			.lock()
			.map_err(|_| PopMetronomeEventError::MutexPoisoned)?
			.pop())
	}
}

impl std::fmt::Debug for MetronomeHandle {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		#[derive(Debug)]
		struct EventConsumer;

		f.debug_struct("MetronomeHandle")
			.field("id", &self.id)
			.field("command_producer", &self.command_producer)
			.field("event_consumer", &EventConsumer)
			.finish()
	}
}
