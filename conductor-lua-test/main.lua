package.cpath = love.filesystem.getWorkingDirectory() .. '/target/release/?.dll'

local conductor = require 'conductor'

local manager = conductor.newManager {
	metronomeSettings = {
		tempo = 150,
		intervalEventsToEmit = {1, .5, .25},
	},
}

function love.update(dt)
	for _, event in ipairs(manager:getEvents()) do
		print(event.event, event.interval)
	end
end

function love.keypressed(key)
	if key == 'space' then
		manager:startMetronome()
	end
	if key == 'escape' then
		love.event.quit()
	end
end
