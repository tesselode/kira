# v0.1.0 beta 2 - January 4, 2022

- Fix streaming sound pause/resume/stop fades never starting when the start time
  is set to a clock time

# v0.1.0 beta 1 - January 3, 2022

- Add `load_from_cursor`/`stream_from_cursor`
- Add a `Tween` argument to `StreamingSoundHandle::set_volume`,
  `set_playback_rate`, and `set_panning`
- Change volume settings to use the `Volume` type
- Change playback rate settings to use the `PlaybackRate` type
- Use dB scale for fades when pausing/resuming/stopping
- Use the 2018 edition

# v0.1.0 beta 0 - December 4, 2021

Initial release
