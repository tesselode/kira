use conductor::{
	duration::Duration,
	instance::{InstanceId, InstanceSettings},
	manager::{AudioManager, AudioManagerSettings},
	metronome::MetronomeSettings,
	sequence::{Sequence, SequenceId},
	sound::{SoundId, SoundMetadata},
	tempo::Tempo,
	tween::Tween,
};
use ggez::{
	event::{KeyCode, KeyMods},
	graphics, Context, GameResult,
};
use std::error::Error;

#[derive(Debug, Copy, Clone)]
enum CustomEvent {
	Test,
}

struct MainState {
	audio_manager: AudioManager<CustomEvent>,
	hat_sequence_id: SequenceId,
	hat_sequence_muted: bool,
}

impl MainState {
	pub fn new() -> Result<Self, Box<dyn Error>> {
		let mut audio_manager = AudioManager::new(AudioManagerSettings {
			metronome_settings: MetronomeSettings {
				tempo: Tempo(128.0),
				interval_events_to_emit: vec![0.25, 0.5, 1.0],
			},
			..Default::default()
		})?;
		let test_loop_sound_id = audio_manager.load_sound(
			&std::env::current_dir()
				.unwrap()
				.join("assets/test_loop.ogg"),
			SoundMetadata {
				tempo: Some(Tempo(128.0)),
			},
		)?;
		let hat_sound_id = audio_manager.load_sound(
			&std::env::current_dir().unwrap().join("assets/cymbal.ogg"),
			SoundMetadata {
				tempo: Some(Tempo(128.0)),
			},
		)?;
		let mut test_loop_sequence = Sequence::new();
		test_loop_sequence.wait_for_interval(1.0);
		test_loop_sequence.play_sound(test_loop_sound_id, InstanceSettings::default());
		test_loop_sequence.wait(Duration::Beats(4.0));
		test_loop_sequence.go_to(1);
		audio_manager.start_sequence(test_loop_sequence)?;
		let mut hat_sequence = Sequence::new();
		hat_sequence.wait_for_interval(1.0);
		hat_sequence.play_sound(hat_sound_id, InstanceSettings::default());
		hat_sequence.wait(Duration::Beats(0.25));
		hat_sequence.go_to(1);
		let hat_sequence_id = audio_manager.start_sequence(hat_sequence)?;
		audio_manager.start_metronome()?;
		Ok(Self {
			audio_manager,
			hat_sequence_id,
			hat_sequence_muted: false,
		})
	}
}

impl ggez::event::EventHandler for MainState {
	fn update(&mut self, _ctx: &mut Context) -> GameResult {
		self.audio_manager.events();
		self.audio_manager.free_unused_resources();
		Ok(())
	}

	fn key_down_event(
		&mut self,
		_ctx: &mut Context,
		keycode: KeyCode,
		_keymods: KeyMods,
		_repeat: bool,
	) {
		match keycode {
			KeyCode::Space => {
				if self.hat_sequence_muted {
					self.audio_manager
						.unmute_sequence(self.hat_sequence_id)
						.unwrap();
				} else {
					self.audio_manager
						.mute_sequence(self.hat_sequence_id)
						.unwrap();
				}
				self.hat_sequence_muted = !self.hat_sequence_muted;
			}
			_ => {}
		}
	}

	fn draw(&mut self, ctx: &mut Context) -> GameResult {
		graphics::clear(ctx, graphics::BLACK);
		graphics::present(ctx)?;
		Ok(())
	}
}

fn main() -> Result<(), Box<dyn Error>> {
	let (mut ctx, mut event_loop) = ggez::ContextBuilder::new("conductor-test", "tesselode")
		.modules(ggez::conf::ModuleConf {
			audio: false,
			..Default::default()
		})
		.build()?;
	let mut main_state = MainState::new()?;
	ggez::event::run(&mut ctx, &mut event_loop, &mut main_state)?;
	Ok(())
}
