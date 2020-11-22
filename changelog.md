# v0.2
- Rename `StereoSample` to `Frame`
- Added support for panning instances:
	- Added `Frame::panned`
	- Added a `panning` field and method to `InstanceSettings`
	- Added `AudioManager::set_instance_panning`
	- Added `Sequence::set_instance_panning`
- Added parameter mappings, which allow `Value`s to map to parameters with
custom scaling and offsets. `Value::Parameter` now contains a `Mapping`
as its second piece of data. `ParameterId`s can be converted into
`Value::Parameter`s with the default 1:1 mapping.
- Changed `Tween` to a C-style struct with named fields:
	- `duration` - the duration of the tween
	- `easing` - the easing function to use (linear, power, etc.)
	- `ease_direction` - the easing direction (in, out, in-out)

# v0.1
First public release
