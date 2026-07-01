-- Hello Event Plugin
-- A simple Lua plugin that uses the player join event

return {
	name = "HelloEvent",
	description = "A simple plugin that demonstrates event handling",
	version = "1.0.0",
	author = "PLua",

	on_enable = function()
		pumpkin.log.info("Hello Event plugin enabled!")

		local listener_id = pumpkin.events.register_listener("player_join", function(event)
			pumpkin.log.info("Player joined: " .. event.player_name)
			pumpkin.server.broadcast_message("Welcome to the server, " .. event.player_name .. "!")
		end)

		_G.player_join_listener = listener_id
		pumpkin.log.info("Registered player_join event listener: " .. listener_id)
	end,

	on_disable = function()
		pumpkin.log.info("Hello Event plugin disabled!")

		if _G.player_join_listener then
			pumpkin.events.unregister_listener("player_join", _G.player_join_listener)
			pumpkin.log.info("Unregistered player_join event listener: " .. _G.player_join_listener)
			_G.player_join_listener = nil
		end
	end,
}
