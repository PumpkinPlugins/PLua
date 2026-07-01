use piccolo::{Executor, Table, Value};
use pumpkin_plugin_api::{
    Server,
    events::{EventData, EventHandler, PlayerJoinEvent},
};

use crate::script::runtime::get_lua;

fn listeners_table<'gc>(ctx: piccolo::Context<'gc>, event_type: &'static str) -> Table<'gc> {
    let pumpkin = ctx.globals().get(ctx, "pumpkin");
    if let Value::Table(p) = pumpkin {
        let events = p.get(ctx, "events");
        if let Value::Table(e) = events {
            let lst = e.get(ctx, event_type);
            if let Value::Table(l) = lst {
                return l;
            }
        }
    }
    Table::new(&ctx)
}

pub struct PlayerJoinEventHandler;

impl EventHandler<PlayerJoinEvent> for PlayerJoinEventHandler {
    fn handle(
        &self,
        _server: Server,
        event: EventData<PlayerJoinEvent>,
    ) -> EventData<PlayerJoinEvent> {
        let Some(lua) = get_lua() else {
            return event;
        };

        let player_name = event.player.get_name();
        let join_message = event.join_message.get_text();

        match lua.try_enter(|ctx| {
            let listeners = listeners_table(ctx, "player_join");

            let event_table = Table::new(&ctx);
            let name_str = piccolo::String::from_slice(&ctx, player_name.as_bytes());
            event_table.set(ctx, "player_name", name_str).ok();
            let msg_str = piccolo::String::from_slice(&ctx, join_message.as_bytes());
            event_table.set(ctx, "join_message", msg_str).ok();
            event_table.set(ctx, "cancelled", event.cancelled).ok();

            let mut stashed = Vec::new();
            for (_, func_val) in listeners.iter() {
                if let Value::Function(func) = func_val {
                    let executor = ctx.stash(Executor::start(ctx, func, (event_table,)));
                    stashed.push(executor);
                }
            }
            Ok(stashed)
        }) {
            Ok(executors) => {
                for executor in &executors {
                    if let Err(e) = lua.execute::<()>(executor) {
                        tracing::error!("PLua event: failed to execute listener: {}", e);
                    }
                }
            }
            Err(e) => {
                tracing::error!("PLua event: try_enter failed: {}", e);
            }
        }

        event
    }
}
