use std::time::UNIX_EPOCH;

use mlua::prelude::*;
use rand::{Rng, rng};

pub struct Events;

impl LuaUserData for Events {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_function(
            "register_listener",
            |lua, (event_type, callback): (String, LuaFunction)| {
                let globals = lua.globals();
                let pumpkin: LuaTable = globals.get("pumpkin")?;
                let events: LuaTable = pumpkin.get("events")?;

                let timestamp = UNIX_EPOCH.elapsed().unwrap_or_default().as_millis();

                let random = rng().random::<u32>();

                let plugin_name = lua
                    .globals()
                    .get::<LuaTable>("PLUGIN_INFO")
                    .and_then(|t| t.get::<String>("name"))
                    .unwrap_or_else(|_| "unknown".to_string());

                let callback_name = callback
                    .info()
                    .name
                    .unwrap_or_else(|| event_type.clone())
                    .replace(|c: char| !c.is_alphanumeric(), "");

                let listener_id = format!(
                    "listener_{}_{}_{}_{}",
                    plugin_name, callback_name, timestamp, random
                );

                match event_type.as_str() {
                    "player_join" => {
                        let listeners: LuaTable = events.get("player_join")?;
                        listeners.set(listener_id.clone(), callback)?;
                        Ok(listener_id)
                    }
                    "player_leave" => {
                        let listeners: LuaTable = events.get("player_leave")?;
                        listeners.set(listener_id.clone(), callback)?;
                        Ok(listener_id)
                    }
                    "player_chat" => {
                        let listeners: LuaTable = events.get("player_chat")?;
                        listeners.set(listener_id.clone(), callback)?;
                        Ok(listener_id)
                    }
                    "block_place" => {
                        let listeners: LuaTable = events.get("block_place")?;
                        listeners.set(listener_id.clone(), callback)?;
                        Ok(listener_id)
                    }
                    "block_break" => {
                        let listeners: LuaTable = events.get("block_break")?;
                        listeners.set(listener_id.clone(), callback)?;
                        Ok(listener_id)
                    }
                    _ => Err(mlua::Error::RuntimeError(format!(
                        "Unknown event type: {}",
                        event_type
                    ))),
                }
            },
        );
        methods.add_function(
            "unregister_listener",
            |lua, (event_type, listener_id): (String, String)| {
                let globals = lua.globals();
                let pumpkin: LuaTable = globals.get("pumpkin")?;
                let events: LuaTable = pumpkin.get("events")?;

                match event_type.as_str() {
                    "player_join" => {
                        let listeners: LuaTable = events.get("player_join")?;
                        listeners.set(listener_id, mlua::Value::Nil)?;
                        Ok(true)
                    }
                    "player_leave" => {
                        let listeners: LuaTable = events.get("player_leave")?;
                        listeners.set(listener_id, mlua::Value::Nil)?;
                        Ok(true)
                    }
                    "player_chat" => {
                        let listeners: LuaTable = events.get("player_chat")?;
                        listeners.set(listener_id, mlua::Value::Nil)?;
                        Ok(true)
                    }
                    "block_place" => {
                        let listeners: LuaTable = events.get("block_place")?;
                        listeners.set(listener_id, mlua::Value::Nil)?;
                        Ok(true)
                    }
                    "block_break" => {
                        let listeners: LuaTable = events.get("block_break")?;
                        listeners.set(listener_id, mlua::Value::Nil)?;
                        Ok(true)
                    }
                    _ => Err(mlua::Error::RuntimeError(format!(
                        "Unknown event type: {}",
                        event_type
                    ))),
                }
            },
        );
    }
}
