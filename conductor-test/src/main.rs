use conductor::{
	manager::{AudioManager, AudioManagerSettings, InstanceHandle, InstanceSettings},
	project::{Project, SoundId, SoundSettings},
};
use ggez::{
	event::{KeyCode, KeyMods},
	graphics, Context, GameResult,
};
use std::error::Error;

struct MainState {
	audio_manager: AudioManager,
	sound_id: SoundId,
	instance_handle: Option<InstanceHandle>,
}

impl MainState {
	pub fn new() -> Result<Self, Box<dyn Error>> {
		let mut project = Project::new();
		let sound_id = project.load_sound(
			&std::env::current_dir().unwrap().join("assets/cymbal.ogg"),
			SoundSettings::default(),
		)?;
		Ok(Self {
			audio_manager: AudioManager::new(project, AudioManagerSettings::default())?,
			sound_id,
			instance_handle: None,
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
			KeyCode::Space => {
				self.instance_handle = Some(
					self.audio_manager
						.play_sound(self.sound_id, InstanceSettings::default())
						.unwrap(),
				);
			}
			KeyCode::P => {
				if let Some(instance_handle) = self.instance_handle {
					self.audio_manager
						.set_instance_volume(instance_handle, 0.5)
						.unwrap();
				}
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
