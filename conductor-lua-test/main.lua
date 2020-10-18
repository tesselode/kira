package.cpath = love.filesystem.getWorkingDirectory() .. '/target/debug/?.dll'

local conductor = require 'conductor'
local manager = conductor.newManager()
local soundId = manager:loadSound 'assets/loop.ogg'
local instanceId = manager:playSound(soundId)

function love.keypressed(key)
	if key == 'space' then
		manager:pauseInstancesOfSound(soundId, .5)
	end
end
