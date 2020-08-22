package.cpath = love.filesystem.getWorkingDirectory() .. '/target/release/?.dll'

local conductor = require 'conductor'

local manager = conductor.newManager {
	metronomeSettings = {
		tempo = 128,
	},
}
local loopSoundId = manager:loadSound 'assets/test_loop.ogg'
local hatSoundId = manager:loadSound 'assets/hhclosed.ogg'
local loopSequence = conductor.newSequence()
loopSequence:waitForInterval(1)
loopSequence:playSound(loopSoundId)
loopSequence:wait(4, 'beats')
loopSequence:goTo(2)
manager:startSequence(loopSequence)
local hatSequence = conductor.newSequence()
hatSequence:waitForInterval(1)
hatSequence:playSound(hatSoundId)
hatSequence:wait(.25, 'beats')
hatSequence:goTo(2)
local hatSequenceId = manager:startSequence(hatSequence)
manager:startMetronome()
local hatSequenceMuted = false

function love.update(dt)
	manager:freeUnusedResources()
	manager:getEvents()
end

function love.keypressed(key)
	if key == 'space' then
		if hatSequenceMuted then
			manager:unmuteSequence(hatSequenceId)
		else
			manager:muteSequence(hatSequenceId)
		end
		hatSequenceMuted = not hatSequenceMuted
	end
	if key == 'escape' then
		love.event.quit()
	end
end
