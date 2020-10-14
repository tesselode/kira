package.cpath = love.filesystem.getWorkingDirectory() .. '/target/debug/?.dll'

local conductor = require 'conductor'

local manager = conductor.newManager {
	metronomeSettings = {
		tempo = 128,
		intervalEventsToEmit = {0.25, 0.5, 1.0},
	},
}
local customEventHandle = conductor.newCustomEventHandle()
local testSequence = conductor.newSequence()
testSequence:wait(1, 'beat')
testSequence:emitCustomEvent(customEventHandle)
testSequence:goTo(1)
manager:startSequence(testSequence)
manager:startMetronome()

local callbacks = {
	metronomeIntervalPassed = function(interval)
		print('interval passed:', interval)
	end,
	custom = function(handle)
		if handle == customEventHandle then
			print 'custom event'
		else
			print 'idk what this is'
		end
	end,
}

function love.update(dt)
	manager:freeUnusedResources()
	manager:getEvents(callbacks)
end

function love.keypressed(key)
	if key == 'escape' then love.event.quit() end
end
