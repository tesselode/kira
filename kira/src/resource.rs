use crate::{
	arrangement::Arrangement,
	audio_stream::AudioStream,
	group::Group,
	metronome::Metronome,
	mixer::{effect_slot::EffectSlot, Track},
	sequence::SequenceInstance,
	sound::Sound,
};

pub(crate) enum Resource {
	Sound(Sound),
	Arrangement(Arrangement),
	SequenceInstance(SequenceInstance),
	Track(Track),
	EffectSlot(EffectSlot),
	Group(Group),
	Stream(Box<dyn AudioStream>),
	Metronome(Metronome),
}
