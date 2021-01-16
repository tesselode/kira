use thiserror::Error;

/// Something that can go wrong.
#[derive(Debug, Error)]
pub enum AudioError {
	#[error("The backend cannot receive commands because it no longer exists")]
	BackendDisconnected,

	#[error(
		"Cannot pop an event from a receiver because the receiver is currently mutably borrowed"
	)]
	EventReceiverBorrowed,
}

/// A wrapper around the standard [`Result`](Result)
/// type that always has an [`AudioError`](AudioError)
/// as its error type.
pub type AudioResult<T> = Result<T, AudioError>;
