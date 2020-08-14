use conductor::{
	duration::Duration,
	instance::{InstanceId, InstanceSettings},
	manager::{AudioManager, AudioManagerSettings},
	metronome::MetronomeSettings,
	sequence::Sequence,
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
	sound_id: SoundId,
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
		let sound_id = audio_manager.load_sound(
			&std::env::current_dir()
				.unwrap()
				.join("assets/test_loop.ogg"),
			SoundMetadata {
				tempo: Some(Tempo(128.0)),
			},
		)?;
		let mut sequence = Sequence::new();
		sequence.wait_for_interval(1.0);
		let handle = sequence.play_sound(sound_id, InstanceSettings::default());
		sequence.wait(Duration::Beats(3.0));
		sequence.set_instance_volume(handle, 0.0, Some(Tween(0.25)));
		sequence.wait(Duration::Beats(0.5));
		sequence.set_instance_volume(handle, 1.0, Some(Tween(0.25)));
		sequence.wait(Duration::Beats(0.5));
		sequence.go_to(1);
		audio_manager.start_sequence(sequence)?;
		audio_manager.start_metronome()?;
		Ok(Self {
			audio_manager,
			sound_id,
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
