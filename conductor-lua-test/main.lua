package.cpath = love.filesystem.getWorkingDirectory() .. '/target/release/?.dll'

local conductor = require 'conductor'

local manager = conductor.newManager()
conductor.test(5, conductor.DURATION_UNIT_BEATS)

function love.keypressed(key)
	if key == 'escape' then
		love.event.quit()
	end
end
