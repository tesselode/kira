package.cpath = love.filesystem.getWorkingDirectory() .. '/target/release/?.dll'

local conductor = require 'conductor'

local manager = conductor.newManager()

local soundId = manager:loadSound('assets/test_loop.ogg', {
	tempo = 128,
})
print(soundId:getMetadata():getTempo())

function love.keypressed(key)
	if key == 'escape' then
		love.event.quit()
	end
end
