package.cpath = love.filesystem.getWorkingDirectory() .. '/target/release/?.dll'

local conductor = require 'conductor'

local manager = conductor.newManager()

local soundId = manager:loadSound 'assets/test_loop.ogg'
local instanceId = manager:playSound(soundId)
manager:setInstancePitch(instanceId, 0.25, {duration = 1})

function love.keypressed(key)
	if key == 'escape' then
		love.event.quit()
	end
end
