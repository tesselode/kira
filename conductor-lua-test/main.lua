package.cpath = love.filesystem.getWorkingDirectory() .. '/target/release/?.dll'

local conductor = require 'conductor'

local manager = conductor.newManager {
	metronomeSettings = {
		tempo = 128,
		intervalEventsToEmit = {0.25, 0.5, 1.0},
	},
}
local hatSoundId = manager:loadSound('assets/hhclosed.ogg', {
	cooldown = 0,
	metadata = {
		tempo = 120,
	}
})
print(hatSoundId:getMetadata():getTempo())
local hatSequence = conductor.newSequence()
hatSequence:playSound(hatSoundId)
hatSequence:playSound(hatSoundId, {pitch = 0.25})
hatSequence:wait(1, 'beats')
hatSequence:goTo(1)
manager:startSequence(hatSequence)
manager:startMetronome()

local callbacks = {
	metronomeIntervalPassed = function(interval)
		print('interval passed:', interval)
	end,
	custom = function(index)
		print('custom event emitted:', index)
	end,
}

function love.update(dt)
	manager:freeUnusedResources()
	manager:getEvents(callbacks)
end

function love.keypressed(key)
	if key == 'escape' then love.event.quit() end
end
