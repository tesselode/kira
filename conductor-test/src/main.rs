use conductor::{manager::AudioManager, project::Project};
use ggez::{graphics, Context, GameResult};
use std::error::Error;

struct MainState {
	audio_manager: AudioManager,
}

impl MainState {
	pub fn new() -> Result<Self, Box<dyn Error>> {
		let mut project = Project::new();
		project.load_sound(&std::env::current_dir().unwrap().join("assets/hhclosed.ogg"))?;
		Ok(Self {
			audio_manager: AudioManager::new(project)?,
		})
	}
}

impl ggez::event::EventHandler for MainState {
	fn update(&mut self, _ctx: &mut Context) -> GameResult {
		Ok(())
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
