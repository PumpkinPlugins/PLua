use pumpkin_plugin_api::{Context, events::EventPriority};

mod block_break;
mod block_place;
mod player_chat;
mod player_join;
mod player_leave;

pub use block_break::BlockBreakEventHandler;
pub use block_place::BlockPlaceEventHandler;
pub use player_chat::PlayerChatEventHandler;
pub use player_join::PlayerJoinEventHandler;
pub use player_leave::PlayerLeaveEventHandler;

pub const ALL_EVENT_TYPES: &[&str] = &[
    "player_join",
    "player_leave",
    "player_chat",
    "block_break",
    "block_place",
];

pub fn register_all_handlers(context: &Context) -> pumpkin_plugin_api::Result<()> {
    context.register_event_handler(PlayerJoinEventHandler, EventPriority::Normal, false)?;

    context.register_event_handler(PlayerLeaveEventHandler, EventPriority::Normal, false)?;

    context.register_event_handler(PlayerChatEventHandler, EventPriority::Normal, false)?;

    context.register_event_handler(BlockBreakEventHandler, EventPriority::Normal, false)?;

    context.register_event_handler(BlockPlaceEventHandler, EventPriority::Normal, false)?;

    Ok(())
}
