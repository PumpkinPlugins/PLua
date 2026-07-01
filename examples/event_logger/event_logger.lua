-- Event Logger Plugin
-- A comprehensive plugin that logs all available events

local listeners = {}

return {
	name = "EventLogger",
	description = "Logs all supported events in the PLua system",
	version = "1.0.0",
	author = "PLua",

	on_enable = function()
		pumpkin.log.info("Event Logger plugin enabled!")

		listeners.player_join = pumpkin.events.register_listener("player_join", function(event)
			pumpkin.log.info("[JOIN] Player " .. event.player_name .. " joined: " .. event.join_message)
		end)

		listeners.player_leave = pumpkin.events.register_listener("player_leave", function(event)
			pumpkin.log.info("[LEAVE] Player " .. event.player_name .. " left: " .. event.leave_message)
		end)

		listeners.player_chat = pumpkin.events.register_listener("player_chat", function(event)
			pumpkin.log.info("[CHAT] " .. event.player_name .. ": " .. event.message)
		end)

		listeners.block_place = pumpkin.events.register_listener("block_place", function(event)
			pumpkin.log.info(
				"[BLOCK_PLACE] Player "
					.. event.player_name
					.. " placed "
					.. event.block_placed
					.. " at ("
					.. event.x
					.. ","
					.. event.y
					.. ","
					.. event.z
					.. ")"
					.. " (can build: "
					.. tostring(event.can_build)
					.. ")"
			)
		end)

		listeners.block_break = pumpkin.events.register_listener("block_break", function(event)
			local player = event.player_name or "unknown"
			pumpkin.log.info(
				"[BLOCK_BREAK] Player "
					.. player
					.. " broke "
					.. event.block
					.. " at ("
					.. event.x
					.. ","
					.. event.y
					.. ","
					.. event.z
					.. ")"
					.. " (exp: "
					.. event.exp
					.. ", drop: "
					.. tostring(event.should_drop)
					.. ")"
			)
		end)

		pumpkin.log.info("All event listeners registered successfully")
		pumpkin.server.broadcast_message("Event Logger is now active - all events will be logged!")
	end,

	on_disable = function()
		pumpkin.log.info("Event Logger plugin disabled!")

		for event_type, listener_id in pairs(listeners) do
			pumpkin.events.unregister_listener(event_type, listener_id)
			pumpkin.log.info("Unregistered " .. event_type .. " listener: " .. listener_id)
		end

		listeners = {}
	end,
}
