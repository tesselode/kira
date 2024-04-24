use triple_buffer::{triple_buffer, Input, Output};

use crate::tween::{Tween, Value};

pub struct CommandWriter<T: Send + Copy>(Input<Option<T>>);

impl<T: Send + Copy> CommandWriter<T> {
	pub fn write(&mut self, command: T) {
		self.0.write(Some(command))
	}
}

pub struct CommandReader<T: Send + Copy>(Output<Option<T>>);

impl<T: Send + Copy> CommandReader<T> {
	#[must_use]
	pub fn read(&mut self) -> Option<T> {
		if self.0.update() {
			*self.0.output_buffer()
		} else {
			None
		}
	}
}

#[must_use]
pub fn command_writer_and_reader<T: Send + Copy>() -> (CommandWriter<T>, CommandReader<T>) {
	let (input, output) = triple_buffer(&None);
	(CommandWriter(input), CommandReader(output))
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ValueChangeCommand<T> {
	pub target: Value<T>,
	pub tween: Tween,
}

#[macro_export]
macro_rules! command_writers_and_readers {
	($($field_name:ident: $type:ty),*$(,)?) => {
		pub(crate) struct CommandWriters {
			$($field_name: $crate::command::CommandWriter<$type>),*
		}

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

#[macro_export]
macro_rules! read_commands_into_parameters {
	($self:ident, $($parameter_name:ident),*$(,)?) => {
		paste::paste! {
			$($self.$parameter_name.read_command(&mut $self.command_readers.[<set_ $parameter_name>]);)*
		}
	};
}

#[macro_export]
macro_rules! handle_param_setters {
	($($(#[$m:meta])* $name:ident: $type:ty),*$(,)?) => {
		paste::paste! {
			$(
				$(#[$m])*
				pub fn [<set_ $name>](&mut self, $name: impl Into<$crate::tween::Value<$type>>, tween: $crate::tween::Tween) {
					self.command_writers.[<set_ $name>].write($crate::command::ValueChangeCommand {
						target: $name.into(),
						tween,
					})
				}
			)*
		}
	};
}
