# PLua - Lua Plugin System for Pumpkin

PLua is a WASM plugin for the Pumpkin Minecraft server that enables loading and managing plugins written in Lua. It embeds a Lua 5.4 runtime (via [piccolo](https://github.com/kyren/piccolo)) and proxies Minecraft events to Lua callbacks.

## Installation

### Pre-built Binary

Download `plua.wasm` from [The Pumpkin Market](https://market.pumpkinmc.org/plugin/44) and place it in your Pumpkin server's plugins directory.

### Building from Source

```bash
git clone https://github.com/PumpkinPlugins/PLua.git
cd PLua

# Add the WASM target
rustup target add wasm32-wasip2

# Build
cargo build --release

# The compiled plugin is at target/wasm32-wasip2/release/plua.wasm
```

## Getting Started

1. Place `plua.wasm` in your Pumpkin server's plugins directory
2. Start the server — PLua will create its data directory with a `plugins/` subfolder
3. Place `.lua` plugin files in `plugins/plua/plugins/` (relative to the server)
4. Use the `/plua` command to manage plugins

## Commands

- `/plua list` — List all available Lua plugins and their status
- `/plua enable <name>` — Enable a plugin
- `/plua disable <name>` — Disable a plugin
- `/plua reload` — Reload all enabled plugins
- `/plua reload <name>` — Reload a specific plugin
- `/plua info <name>` — Show plugin metadata

Requires OP level 4 (`plua:command.plua` permission).

## Writing Lua Plugins

A plugin is a `.lua` file that returns a table with metadata and optional lifecycle hooks:

```lua
return {
    name = "MyPlugin",
    description = "Does something useful",
    version = "1.0.0",
    author = "Your Name",

    on_enable = function()
        pumpkin.log.info("MyPlugin enabled!")
    end,

    on_disable = function()
        pumpkin.log.info("MyPlugin disabled!")
    end,
}
```

The `on_enable` function is called when the plugin is loaded. The `on_disable` function is called when unloaded. Both are optional.

## API Reference

PLua exposes a global `pumpkin` table:

### Logging

```lua
pumpkin.log.info("message")
pumpkin.log.warn("message")
pumpkin.log.error("message")
pumpkin.log.debug("message")
```

### Server

```lua
pumpkin.server.broadcast_message("Hello everyone!")
```

### Events

```lua
-- Register a listener (returns a listener ID)
local id = pumpkin.events.register_listener("player_join", function(event)
    pumpkin.log.info(event.player_name .. " joined!")
end)

-- Unregister later
pumpkin.events.unregister_listener("player_join", id)
```

## Supported Events

### player_join

| Field | Type | Description |
|---|---|---|
| `player_name` | string | The joining player's name |
| `join_message` | string | The join message |
| `cancelled` | boolean | Whether the event was cancelled |

### player_leave

| Field | Type | Description |
|---|---|---|
| `player_name` | string | The leaving player's name |
| `leave_message` | string | The leave message |
| `cancelled` | boolean | Whether the event was cancelled |

### player_chat

| Field | Type | Description |
|---|---|---|
| `player_name` | string | The player's name |
| `message` | string | The chat message content |
| `cancelled` | boolean | Whether the event was cancelled |

### block_place

| Field | Type | Description |
|---|---|---|
| `player_name` | string | The player's name |
| `block_placed` | string | The block being placed |
| `x`, `y`, `z` | integer | Block position |
| `can_build` | boolean | Whether player can build here |
| `cancelled` | boolean | Whether the event was cancelled |

### block_break

| Field | Type | Description |
|---|---|---|
| `player_name` | string | The player's name (empty if no player) |
| `block` | string | The block type broken |
| `x`, `y`, `z` | integer | Block position |
| `exp` | integer | Experience to drop |
| `should_drop` | boolean | Whether items should drop |
| `cancelled` | boolean | Whether the event was cancelled |

## Notes

- PLua uses **Lua 5.4** (via piccolo). Luau-specific syntax like `--!strict` and type annotations are **not supported**.
- Plugins are loaded from the `plugins/` subdirectory inside PLua's data folder.
- Plugin files must have a `.lua` or `.luau` extension.
- The `string` standard library is limited; string concatenation (`..`) works, but `string.format` is not available.

## License

MIT — same as the Pumpkin server.
