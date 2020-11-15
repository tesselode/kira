package.cpath = love.filesystem.getWorkingDirectory() .. '/target/debug/?.dll'

local kira = require 'kira'

local manager = kira.newManager()
local parameterId = manager:addParameter(1)
local soundId = manager:loadSound('assets/loop.ogg', {
	metadata = {
		semanticDuration = (60 / 128) * 16,
	},
})
manager:playSound(soundId, {pitch = parameterId, loop = true})
manager:setParameter(parameterId, .25, 5)
