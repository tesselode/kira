package.cpath = love.filesystem.getWorkingDirectory() .. '/target/debug/?.dll'

local conductor = require 'conductor'
local manager = conductor.newManager()
local soundId = manager:loadSound('assets/loop.ogg', {
	metadata = {
		semanticDuration = {1, 'beat'},
	},
})
print(manager:playSound(soundId, {pitch = .5, fadeInDuration = .25}))
