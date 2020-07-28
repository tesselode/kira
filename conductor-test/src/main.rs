use conductor::{
	sound_bank::{SoundBank, SoundId},
	AudioManager,
};
use ggez::{
	event::{KeyCode, KeyMods},
	graphics, Context, GameResult,
};
use std::error::Error;

pub struct MainState {
	sound_id: SoundId,
	audio_manager: AudioManager,
}

impl MainState {
	pub fn new() -> Result<Self, Box<dyn Error>> {
		let mut sound_bank = SoundBank::new();
		let sound_id =
			sound_bank.load(&std::env::current_dir().unwrap().join("assets/cymbal.ogg"))?;
		Ok(Self {
			sound_id,
			audio_manager: AudioManager::new(sound_bank)?,
		})
	}
}

impl ggez::event::EventHandler for MainState {
	fn update(&mut self, _ctx: &mut Context) -> GameResult {
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
			KeyCode::Space => self.audio_manager.play_sound(self.sound_id),
			_ => {}
		}
	}

	fn draw(&mut self, ctx: &mut Context) -> GameResult {
		graphics::clear(ctx, graphics::BLACK);
		let text = graphics::Text::new("hi!");
		graphics::draw(ctx, &text, graphics::DrawParam::new())?;
		graphics::present(ctx)?;
		Ok(())
	}
}

fn main() -> GameResult {
	let (mut ctx, mut event_loop) = ggez::ContextBuilder::new("conductor-test", "tesselode")
		.modules(ggez::conf::ModuleConf {
			audio: false,
			..Default::default()
		})
		.build()?;
	let mut main_state = MainState::new().unwrap();
	ggez::event::run(&mut ctx, &mut event_loop, &mut main_state)?;
	Ok(())
}
