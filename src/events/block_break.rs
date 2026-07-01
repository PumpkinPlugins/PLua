use piccolo::{Executor, Table, Value};
use pumpkin_plugin_api::{
    Server,
    events::{BlockBreakEvent, EventData, EventHandler},
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

pub struct BlockBreakEventHandler;

impl EventHandler<BlockBreakEvent> for BlockBreakEventHandler {
    fn handle(
        &self,
        _server: Server,
        event: EventData<BlockBreakEvent>,
    ) -> EventData<BlockBreakEvent> {
        let Some(lua) = get_lua() else {
            return event;
        };

        let player_name = event.player.as_ref().map(|p| p.get_name()).unwrap_or_default();
        let block = event.block.clone();
        let x = event.block_pos.x;
        let y = event.block_pos.y;
        let z = event.block_pos.z;
        let exp = event.exp;
        let should_drop = event.should_drop;

        match lua.try_enter(|ctx| {
            let listeners = listeners_table(ctx, "block_break");

            let event_table = Table::new(&ctx);
            let name_str = piccolo::String::from_slice(&ctx, player_name.as_bytes());
            event_table.set(ctx, "player_name", name_str).ok();
            let block_str = piccolo::String::from_slice(&ctx, block.as_bytes());
            event_table.set(ctx, "block", block_str).ok();
            event_table.set(ctx, "x", x as i64).ok();
            event_table.set(ctx, "y", y as i64).ok();
            event_table.set(ctx, "z", z as i64).ok();
            event_table.set(ctx, "exp", exp as i64).ok();
            event_table.set(ctx, "should_drop", should_drop).ok();
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
                        tracing::error!("PLua event: failed to execute block_break listener: {}", e);
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
