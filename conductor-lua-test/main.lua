package.cpath = love.filesystem.getWorkingDirectory() .. '/target/debug/?.dll'

local conductor = require 'conductor'
conductor.newManager {
	metronomeSettings = {
		tempo = 'asdf',
	},
}
