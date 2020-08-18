package.cpath = love.filesystem.getWorkingDirectory() .. '/target/release/?.dll'

local conductor = require 'conductor'

local manager = conductor.newManager()
local sound = manager:loadSound 'assets/test_loop.ogg'
print(manager:playSound(sound, {
	fadeInDuration = 1,
}))

function love.keypressed(key)
	if key == 'escape' then
		love.event.quit()
	end
end
