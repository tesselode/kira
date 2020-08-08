use crate::{
	id::{InstanceId, MetronomeId, SoundId},
	manager::InstanceSettings,
};

#[derive(Copy, Clone)]
pub enum Command {
	PlaySound(SoundId, InstanceId, InstanceSettings),
	StartMetronome(MetronomeId),
	PauseMetronome(MetronomeId),
	StopMetronome(MetronomeId),
}
