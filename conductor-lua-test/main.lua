package.cpath = love.filesystem.getWorkingDirectory() .. '/target/debug/?.dll'

local conductor = require 'conductor'

local manager = conductor.newManager {
	metronomeSettings = {
		intervalEventsToEmit = {.25, .5, 1},
	}
}
local customEvent = conductor.newCustomEvent()
local soundId = manager:loadSound('assets/loop.ogg', {
	metadata = {
		tempo = 128,
		semanticDuration = {16, 'beats'},
	},
})
manager:loopSound(soundId)
manager:setMetronomeTempo(128)
manager:startMetronome()

function love.update(dt)
	manager:getEvents {
		metronomeIntervalPassed = function(...) print('metronomeIntervalPassed', ...) end,
		custom = function(event) print('custom', event == customEvent) end,
	}
end
