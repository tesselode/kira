/*!
Helpers for sending commands from the gameplay thread to the audio thread.
You'll only need to use this if you're making your own implementation of
[`Sound`](crate::sound::Sound), [`Effect`](crate::effect::Effect),
or [`Modulator`](crate::modulator::Modulator).

This module provides a [`CommandWriter`] and [`CommandReader`] that can
transfer data from one thread to another without blocking or waiting on either
thread. `CommandReader::read` will return the latest data that was written to
the corresponding `CommandWriter` or return `None` if no new data was written
since the last read.

Keep in mind that if multiple values are written to a `CommandWriter`, any newer
values will overwrite older values that haven't been read yet. Therefore, this
is only suitable when the reader only cares about the most recent value that
has been written; i.e. new values supercede all older values. If you need a
realtime-safe FIFO queue of multiple values, consider using a ring buffer, such as
[`RingBuffer`](rtrb::RingBuffer) from the [rtrb](https://crates.io/crates/rtrb) crate,
which Kira uses internally.
*/

use triple_buffer::{triple_buffer, Input, Output};

use crate::{Tween, Value};

/** Writes values that can be sent to a [`CommandReader`]. */
#[derive(Debug)]
pub struct CommandWriter<T: Send + Copy>(Input<Option<T>>);

impl<T: Send + Copy> CommandWriter<T> {
	/** Writes a new value, overwriting any previous values. */
	pub fn write(&mut self, command: T) {
		self.0.write(Some(command))
	}
}

/** Reads values that were written to a [`CommandWriter`]. */
#[derive(Debug)]
pub struct CommandReader<T: Send + Copy>(Output<Option<T>>);

impl<T: Send + Copy> CommandReader<T> {
	/**
	 * Returns the latest value that was written to the [`CommandWriter`],
	 * or `None` if no new values were written since the last read.
	 */
	#[must_use]
	pub fn read(&mut self) -> Option<T> {
		if self.0.update() {
			*self.0.output_buffer()
		} else {
			None
		}
	}
}

/** Creates a command writer/reader pair. */
#[must_use]
pub fn command_writer_and_reader<T: Send + Copy>() -> (CommandWriter<T>, CommandReader<T>) {
	let (input, output) = triple_buffer(&None);
	(CommandWriter(input), CommandReader(output))
}

/**
 * A command that holds a target [`Value`] and a [`Tween`].
 *
 * Setting something to a [`Value`] with a given [`Tween`] is a common
 * pattern in Kira.
 *
 * `CommandReader<ValueChangeCommand>`s can be passed to [`Parameter`](crate::Parameter)s
 * to quickly set the parameter to a new value read from the [`CommandReader`].
 */
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ValueChangeCommand<T> {
	/// The new value to set something to.
	pub target: Value<T>,
	/// The tween to use to smoothly transition the value.
	pub tween: Tween,
}

/**
Creates a set of command writers and readers and a constructor for them.
You'll only need to use this if you're making your own implementation of
[`Sound`](crate::sound::Sound), [`Effect`](crate::effect::Effect),
or [`Modulator`](crate::modulator::Modulator).

# Example

This macro call...

```ignore
command_writers_and_readers! {
	set_phase: f64,
	set_frequency: ValueChangeCommand<f64>,
}
```

...will produce this code:

```ignore
pub(crate) struct CommandWriters {
	set_phase: kira::command::CommandWriter<f64>,
	set_frequency: kira::command::CommandWriter<ValueChangeCommand<f64>>,
}

pub(crate) struct CommandReaders {
	set_phase: kira::command::CommandReader<f64>,
	set_frequency: kira::command::CommandReader<ValueChangeCommand<f64>>,
}

#[must_use]
pub(crate) fn command_writers_and_readers() -> (CommandWriters, CommandReaders) {
	let (set_phase_writer, set_phase_reader) = kira::command::command_writer_and_reader();
	let (set_frequency_writer, set_frequency_reader) = kira::command::command_writer_and_reader();
	let command_writers = CommandWriters {
		set_phase: set_phase_writer,
		set_frequency: set_frequency_writer,
	};
	let command_readers = CommandReaders {
		set_phase: set_phase_reader,
		set_frequency: set_frequency_reader,
	};
	(command_writers, command_readers)
}
```
*/
#[macro_export]
macro_rules! command_writers_and_readers {
	($($field_name:ident: $type:ty),*$(,)?) => {
		#[derive(Debug)]
		pub(crate) struct CommandWriters {
			$($field_name: $crate::command::CommandWriter<$type>),*
		}

		#[derive(Debug)]
		pub(crate) struct CommandReaders {
			$($field_name: $crate::command::CommandReader<$type>),*
		}

		#[must_use]
		pub(crate) fn command_writers_and_readers() -> (CommandWriters, CommandReaders) {
			paste::paste! {
				$(let ([<$field_name _writer>], [<$field_name _reader>]) = $crate::command::command_writer_and_reader();)*
				let command_writers = CommandWriters {
					$($field_name: [<$field_name _writer>]),*
				};
				let command_readers = CommandReaders {
					$($field_name: [<$field_name _reader>]),*
				};
				(command_writers, command_readers)
			}
		}
	};
}

macro_rules! read_commands_into_parameters {
	($self:ident, $($parameter_name:ident),*$(,)?) => {
		paste::paste! {
			$($self.$parameter_name.read_command(&mut $self.command_readers.[<set_ $parameter_name>]);)*
		}
	};
}

macro_rules! handle_param_setters {
	($($(#[$m:meta])* $name:ident: $type:ty),*$(,)?) => {
		paste::paste! {
		$(
				$(#[$m])*
				pub fn [<set_ $name>](&mut self, $name: impl Into<$crate::Value<$type>>, tween: $crate::tween::Tween) {
					self.command_writers.[<set_ $name>].write($crate::command::ValueChangeCommand {
						target: $name.into(),
						tween,
					})
				}
			)*
		}
	};
}

pub(crate) use handle_param_setters;
pub(crate) use read_commands_into_parameters;
