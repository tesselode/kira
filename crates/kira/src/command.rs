use triple_buffer::{triple_buffer, Input, Output};

use crate::tween::{Tween, Value};

pub struct CommandWriter<T: Send + Clone>(Input<Option<T>>);

impl<T: Send + Clone> CommandWriter<T> {
	pub fn write(&mut self, command: T) {
		self.0.write(Some(command))
	}
}

pub struct CommandReader<T: Send + Clone>(Output<Option<T>>);

impl<T: Send + Clone> CommandReader<T> {
	pub fn read(&mut self) -> Option<&T> {
		if self.0.update() {
			self.0.output_buffer().as_ref()
		} else {
			None
		}
	}
}

pub fn command_writer_and_reader<T: Send + Clone>() -> (CommandWriter<T>, CommandReader<T>) {
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
	{$($field_name:ident: $type:ty),*} => {
		paste::paste! {
			pub(crate) struct CommandWriters {
				$($field_name: $crate::command::CommandWriter<$type>),*
			}

			pub(crate) struct CommandReaders {
				$($field_name: $crate::command::CommandReader<$type>),*
			}

			pub(crate) fn command_writers_and_readers() -> (CommandWriters, CommandReaders) {
				$(let ([<$field_name _writer>], [<$field_name _reader>]) = $crate::command::command_writer_and_reader();)*

				(
					CommandWriters {
						$($field_name: [<$field_name _writer>]),*
					},
					CommandReaders {
						$($field_name: [<$field_name _reader>]),*
					},
				)
			}
		}
	};
}
