package.cpath = love.filesystem.getWorkingDirectory() .. '/target/debug/?.dll'

local conductor = require 'conductor'
local inspect = require 'inspect'

local manager = conductor.newManager {
	metronomeSettings = {
		intervalEventsToEmit = {.25, .5, 1},
	}
}
manager:setMetronomeTempo(128)
manager:startMetronome()

function love.update(dt)
	print(inspect(manager:getEvents()))
end
