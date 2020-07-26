use conductor::AudioManager;
use ggez::{graphics, Context, GameResult};
use std::error::Error;

pub struct MainState {
	audio_manager: AudioManager,
}

impl MainState {
	pub fn new() -> Result<Self, Box<dyn Error>> {
		Ok(Self {
			audio_manager: AudioManager::new()?,
		})
	}
}

impl ggez::event::EventHandler for MainState {
	fn update(&mut self, _ctx: &mut Context) -> GameResult {
		Ok(())
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
