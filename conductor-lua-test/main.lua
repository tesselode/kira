package.cpath = love.filesystem.getWorkingDirectory() .. '/target/release/?.dll'

local conductor = require 'conductor'

local manager = conductor.new_manager()
print(manager.load_sound('assets/test_loop.ogg').duration)

function love.keypressed(key)
	if key == 'escape' then
		love.event.quit()
	end
end
