package.cpath = love.filesystem.getWorkingDirectory() .. '/target/debug/?.dll'

local conductor = require 'conductor'
local customEvent1 = conductor.newCustomEvent()
local customEvent2 = 'asdf'
print(customEvent1 == nil)
