package.cpath = love.filesystem.getWorkingDirectory() .. '/target/release/?.dll'

local conductor = require 'conductor'

local manager = conductor.newManager {
	metronomeSettings = {
		tempo = 128,
	},
}
local soundId = manager:loadSound 'assets/test_loop.ogg'
local sequence = conductor.newSequence()
sequence:waitForInterval(1)
local handle = sequence:playSound(soundId)
sequence:wait(3, 'beats')
sequence:setInstanceVolume(handle, 0, {duration = .25})
sequence:wait(.5, 'beats')
sequence:setInstanceVolume(handle, 1, {duration = .25})
sequence:wait(.5, 'beats')
sequence:goTo(2)
manager:startSequence(sequence)

function love.update(dt)
	manager:freeUnusedResources()
	manager:getEvents()
end

function love.keypressed(key)
	if key == 'space' then
		manager:startMetronome()
	end
	if key == 's' then manager:setMetronomeTempo(150) end
	if key == 'escape' then
		love.event.quit()
	end
end
