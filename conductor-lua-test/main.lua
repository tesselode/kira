package.cpath = love.filesystem.getWorkingDirectory() .. '/target/debug/?.dll'

local conductor = require 'conductor'

local manager = conductor.newManager()
local soundId = manager:loadSound('assets/loop.ogg', {
	metadata = {
		tempo = 128,
		semanticDuration = {16, 'beats'},
	},
})
manager:loopSound(soundId, {
	startPoint = {8, 'beats'},
})

local callbacks = {
	metronomeIntervalPassed = function(interval)
		print('interval passed:', interval)
	end,
	custom = function(handle)
		print('custom event:', handle)
	end,
}

function love.update(dt)
	manager:freeUnusedResources()
	manager:getEvents(callbacks)
end

function love.keypressed(key)
	if key == 'escape' then love.event.quit() end
end
