use mlua::prelude::*;

pub struct Log;

impl LuaUserData for Log {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_function("info", |_, message: String| {
            log::info!("[Lua] {}", message);
            Ok(())
        });
        methods.add_function("warn", |_, message: String| {
            log::warn!("[Lua] {}", message);
            Ok(())
        });
        methods.add_function("error", |_, message: String| {
            log::error!("[Lua] {}", message);
            Ok(())
        });
        methods.add_function("debug", |_, message: String| {
            log::debug!("[Lua] {}", message);
            Ok(())
        });
    }
}
