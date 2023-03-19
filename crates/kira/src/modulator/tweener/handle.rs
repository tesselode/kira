use crate::modulator::ModulatorId;

pub struct TweenerHandle {
	pub(super) id: ModulatorId,
}

impl From<&TweenerHandle> for ModulatorId {
	fn from(handle: &TweenerHandle) -> Self {
		handle.id
	}
}
