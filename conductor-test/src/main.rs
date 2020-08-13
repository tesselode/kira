use conductor::{
	instance::{InstanceId, InstanceSettings},
	manager::{AudioManager, AudioManagerSettings},
	sound::{SoundId, SoundMetadata},
	tempo::Tempo,
	tween::Tween,
};
use ggez::{
	event::{KeyCode, KeyMods},
	graphics, Context, GameResult,
};
use std::error::Error;

struct MainState {
	audio_manager: AudioManager,
	sound_id_1: SoundId,
	sound_id_2: SoundId,
}

impl MainState {
	pub fn new() -> Result<Self, Box<dyn Error>> {
		let mut audio_manager = AudioManager::new(AudioManagerSettings::default())?;
		let sound_id_1 = audio_manager.load_sound(
			&std::env::current_dir()
				.unwrap()
				.join("assets/test_loop.ogg"),
			SoundMetadata {
				tempo: Some(Tempo(128.0)),
			},
		)?;
		let sound_id_2 = audio_manager.load_sound(
			&std::env::current_dir()
				.unwrap()
				.join("assets/test_song.ogg"),
			SoundMetadata::default(),
		)?;
		audio_manager.play_sound(sound_id_1, InstanceSettings::default())?;
		audio_manager.play_sound(sound_id_2, InstanceSettings::default())?;
		Ok(Self {
			audio_manager,
			sound_id_1,
			sound_id_2,
		})
	}
}

impl ggez::event::EventHandler for MainState {
	fn update(&mut self, _ctx: &mut Context) -> GameResult {
		self.audio_manager.events();
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
				self.audio_manager.unload_sound(self.sound_id_2).unwrap();
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
