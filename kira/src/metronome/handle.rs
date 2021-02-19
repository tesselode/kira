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
	/// A thread panicked while using the event receiver.
	#[error("The event receiver cannot be used because a thread panicked while borrowing it.")]
	MutexPoisoned,
}

#[derive(Clone)]
/// Allows you to control a metronome.

// TODO: add a manual impl of Debug
pub struct MetronomeHandle {
	id: MetronomeId,
	command_sender: CommandProducer,
	event_receiver: Arc<Mutex<Consumer<f64>>>,
}

impl MetronomeHandle {
	pub(crate) fn new(
		id: MetronomeId,
		command_sender: CommandProducer,
		event_receiver: Consumer<f64>,
	) -> Self {
		Self {
			id,
			command_sender,
			event_receiver: Arc::new(Mutex::new(event_receiver)),
		}
	}

	/// Gets the ID of the metronome.
	pub fn id(&self) -> MetronomeId {
		self.id
	}

	/// Sets the tempo of the metronome (in beats per minute).
	pub fn set_tempo(&mut self, tempo: impl Into<Value<Tempo>>) -> Result<(), CommandError> {
		self.command_sender
			.push(MetronomeCommand::SetMetronomeTempo(self.id(), tempo.into()).into())
	}

	/// Starts the metronome.
	pub fn start(&mut self) -> Result<(), CommandError> {
		self.command_sender
			.push(MetronomeCommand::StartMetronome(self.id()).into())
	}

	/// Pauses the metronome.
	pub fn pause(&mut self) -> Result<(), CommandError> {
		self.command_sender
			.push(MetronomeCommand::PauseMetronome(self.id()).into())
	}

	/// Stops the metronome and resets its time to zero.
	pub fn stop(&mut self) -> Result<(), CommandError> {
		self.command_sender
			.push(MetronomeCommand::StopMetronome(self.id()).into())
	}

	/// Gets the first interval event that was emitted by this
	/// metronome since the last call to `pop_event`.
	pub fn pop_event(&mut self) -> Result<Option<f64>, PopMetronomeEventError> {
		Ok(self
			.event_receiver
			.lock()
			.map_err(|_| PopMetronomeEventError::MutexPoisoned)?
			.pop())
	}
}
