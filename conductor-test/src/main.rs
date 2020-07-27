use conductor::AudioManager;
use ggez::{graphics, Context, GameResult};
use std::error::Error;

#[derive(Eq, PartialEq, Hash)]
enum SoundName {
	Test,
}

pub struct MainState {
	audio_manager: AudioManager<SoundName>,
}

impl MainState {
	pub fn new() -> Result<Self, Box<dyn Error>> {
		let mut audio_manager = AudioManager::new()?;
		audio_manager.load_sound(
			SoundName::Test,
			&std::env::current_dir()
				.unwrap()
				.join("assets/hhclosed_stereo.ogg"),
		)?;
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
