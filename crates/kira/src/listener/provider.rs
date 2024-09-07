//! Contains types for reporting values of listeners.
//!
//! You'll only need these types if you're creating implementations
//! of the [`Sound`](crate::sound::Sound) or
//! [`Effect`](crate::effect::Effect) traits.

use glam::Vec3;

use crate::arena::{error::ArenaFull, Arena};

use super::{Listener, ListenerId};

/// Provides information about any listener that currently exists.
pub struct ListenerInfoProvider<'a> {
	spatial_track_position: Option<Vec3>,
	kind: ListenerInfoProviderKind<'a>,
}

impl<'a> ListenerInfoProvider<'a> {
	#[must_use]
	pub(crate) fn new(
		spatial_track_position: Option<Vec3>,
		listeners: &'a Arena<Listener>,
	) -> Self {
		Self {
			spatial_track_position,
			kind: ListenerInfoProviderKind::Normal { listeners },
		}
	}

	pub fn spatial_track_position(&self) -> Option<Vec3> {
		self.spatial_track_position
	}

	/// Gets information about the listener with the given ID if it
	/// exists, returns `None` otherwise.
	#[must_use]
	pub fn get(&self, id: ListenerId) -> Option<ListenerInfo> {
		match &self.kind {
			ListenerInfoProviderKind::Normal { listeners } => {
				listeners.get(id.0).map(|listener| ListenerInfo {
					position: listener.position.value().into(),
					orientation: listener.orientation.value().into(),
				})
			}
			ListenerInfoProviderKind::Mock { listener_info } => listener_info.get(id.0).copied(),
		}
	}

	pub fn listener_distance(&self, id: ListenerId) -> Option<f32> {
		self.spatial_track_position.zip(self.get(id)).map(
			|(spatial_track_position, listener_info)| {
				Vec3::from(listener_info.position).distance(spatial_track_position)
			},
		)
	}
}

enum ListenerInfoProviderKind<'a> {
	Normal { listeners: &'a Arena<Listener> },
	Mock { listener_info: Arena<ListenerInfo> },
}

/// Builds a `ListenerValueProvider` that provides fake listener values.
///
/// This is mainly useful for writing tests for implementations
/// of the [`Sound`](crate::sound::Sound) and
/// [`Effect`](crate::effect::Effect) traits.
pub struct MockListenerInfoProviderBuilder {
	spatial_track_position: Option<Vec3>,
	listener_info: Arena<ListenerInfo>,
}

impl MockListenerInfoProviderBuilder {
	/// Creates a new [`MockListenerInfoProviderBuilder`] with room for
	/// the specified number of listeners.
	#[must_use]
	pub fn new(spatial_track_position: Option<Vec3>, capacity: u16) -> Self {
		Self {
			spatial_track_position,
			listener_info: Arena::new(capacity),
		}
	}

	/// Adds a new fake listener to the builder and returns the corresponding
	/// [`ListenerId`].
	pub fn add(&mut self, value: ListenerInfo) -> Result<ListenerId, ArenaFull> {
		Ok(ListenerId(self.listener_info.insert(value)?))
	}

	/// Consumes the builder and returns a [`ListenerValueProvider`].
	#[must_use]
	pub fn build(self) -> ListenerInfoProvider<'static> {
		ListenerInfoProvider {
			spatial_track_position: self.spatial_track_position,
			kind: ListenerInfoProviderKind::Mock {
				listener_info: self.listener_info,
			},
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ListenerInfo {
	pub position: mint::Vector3<f32>,
	pub orientation: mint::Quaternion<f32>,
}
