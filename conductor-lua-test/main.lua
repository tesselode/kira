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
	local events = manager:getEvents()
	for _, event in ipairs(events) do
		if event.kind == 'metronomeIntervalPassed' then
			print('metronomeIntervalPassed', event.interval)
		elseif event.kind == 'custom' then
			print('custom', event.event == customEvent)
		end
	end
end
