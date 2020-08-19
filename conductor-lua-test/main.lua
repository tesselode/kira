package.cpath = love.filesystem.getWorkingDirectory() .. '/target/release/?.dll'

local conductor = require 'conductor'

local manager = conductor.newManager()

local soundId = manager:loadSound 'assets/test_loop.ogg'

local sequence = conductor.newSequence()
sequence:wait(1, 'seconds')
sequence:playSound(soundId)
manager:startSequence(sequence)

function love.keypressed(key)
	if key == 'escape' then
		love.event.quit()
	end
end
