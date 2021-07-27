## Things I don't like about Kira v0.5.x

### Sequences
- Since sequences have a method for every action they can script,
they have a lot of methods. This has some downsides:
  - The number of methods can be overwhelming for new users
  reading the documentation
  - Sequences add a lot to the code size of Kira - sequence-related
  functionality takes up about 1k lines of code
- Sequences encourage you to use the audio thread to provide timing
info for gameplay. I'm not sure whether this is a good practice or
not.
- Sequences are really useful for AetherBeats, but they have a pretty
niche purpose. While I do think many games could benefit from being
able to queue up audio actions with precise timing, a full-blown
scripting system is probably overkill for most games. There's a lot of
API real estate devoted to something most games won't use. It may
be better to provide a simpler, lower-level tool for queueing actions.

#### Possible alternatives
- Most functions could provide a `delay` setting that lets you choose
to wait for a metronome interval or a `Duration` before the action
is performed. You wouldn't be able to set up complex scripting within
Kira itself, but you may be able to get away with queueing things
up from your gameplay code a little bit before they need to happen.
- Sequences could be simplified to only fire "triggers" that other
objects could receive and respond to. Sequences would still be a
complex scripting tool, but the list of things that a sequence can
trigger would be moved out of the sequence impl itself.

### Groups
- Groups are a blanket category for many disparate things (currently:
sounds, arrangements, and sequences. They should apply to metronomes
as well, I just forgot to implement that). All three of those things
can be paused, resumed, and stopped, but if groups expand to cover
more things, that may not hold true. For example, what would it mean
to "stop" a parameter?
- You have to remember to manually add groups to all of the appropriate
sounds and sequences.

#### Possible alternatives
- Nested `AudioManager`s/`Backend`s
  - Nested `Backend`s could be routed to a mixer track of the parent
  `Backend`
  - Nested `AudioManager`s would have the same API as the main one
  - You can remove a lot of resources at once
  - "Pausing" an `AudioManager` would just mean stopping everything
  that updates - no ambiguity
