pub use mlua::prelude::*;
use pumpkin_util::text::TextComponent;

use crate::SERVER;

pub struct Server;

impl LuaUserData for Server {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_async_function("broadcast_message", async |_, message: String| {
            if let Some(server) = SERVER.get() {
                for p in server.get_all_players().await {
                    p.send_system_message(&TextComponent::text(message.clone()))
                        .await;
                }
            }
            Ok(())
        });
    }
}
