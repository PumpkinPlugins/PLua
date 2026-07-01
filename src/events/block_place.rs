use piccolo::{Executor, Table, Value};
use pumpkin_plugin_api::{
    Server,
    events::{BlockPlaceEvent, EventData, EventHandler},
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

pub struct BlockPlaceEventHandler;

impl EventHandler<BlockPlaceEvent> for BlockPlaceEventHandler {
    fn handle(
        &self,
        _server: Server,
        event: EventData<BlockPlaceEvent>,
    ) -> EventData<BlockPlaceEvent> {
        let Some(lua) = get_lua() else {
            return event;
        };

        let player_name = event.player.get_name();
        let block_placed = event.block_placed.clone();
        let x = event.block_pos.x;
        let y = event.block_pos.y;
        let z = event.block_pos.z;
        let can_build = event.can_build;

        match lua.try_enter(|ctx| {
            let listeners = listeners_table(ctx, "block_place");

            let event_table = Table::new(&ctx);
            let name_str = piccolo::String::from_slice(&ctx, player_name.as_bytes());
            event_table.set(ctx, "player_name", name_str).ok();
            let block_str = piccolo::String::from_slice(&ctx, block_placed.as_bytes());
            event_table.set(ctx, "block_placed", block_str).ok();
            event_table.set(ctx, "x", x as i64).ok();
            event_table.set(ctx, "y", y as i64).ok();
            event_table.set(ctx, "z", z as i64).ok();
            event_table.set(ctx, "can_build", can_build).ok();
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
                        tracing::error!("PLua event: failed to execute block_place listener: {}", e);
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
