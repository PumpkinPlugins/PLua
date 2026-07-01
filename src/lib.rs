use std::cell::RefCell;

use pumpkin_plugin_api::{Context, Plugin, PluginMetadata, permissions};

use crate::config::PLuaConfig;
use crate::script::runtime::PluginRuntime;

pub mod commands;
pub mod config;
pub mod events;
pub mod script;

thread_local! {
    pub static SCRIPT_RUNTIME: RefCell<Option<PluginRuntime>> = const { RefCell::new(None) };
    pub static CONFIG: RefCell<PLuaConfig> = RefCell::new(PLuaConfig::default());
}

struct PLuaPlugin;

impl Plugin for PLuaPlugin {
    fn new() -> Self {
        PLuaPlugin
    }

    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "plua".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            authors: vec!["vyPal".into()],
            description: "Write plugins for the Pumpkin MC Server software in Lua".into(),
            dependencies: vec![],
            permissions: vec![
                permissions::FS_READ_DATA.into(),
                permissions::FS_WRITE_DATA.into(),
            ],
        }
    }

    fn on_load(&mut self, context: Context) -> pumpkin_plugin_api::Result<()> {
        let data_folder = context.get_data_folder();

        let config = PLuaConfig::load(&data_folder);
        let mut runtime = PluginRuntime::new(&data_folder);

        let server = context.get_server();
        runtime.init_api(server);

        events::register_all_handlers(&context)?;

        runtime.discover_plugins();
        runtime.load_enabled_plugins(&config);

        config.write(&data_folder);

        SCRIPT_RUNTIME.with(|r| {
            *r.borrow_mut() = Some(runtime);
        });
        CONFIG.with(|c| {
            *c.borrow_mut() = config;
        });

        commands::plua::register(&context)?;

        tracing::info!("PLua loaded successfully!");
        Ok(())
    }

    fn on_unload(&mut self, context: Context) -> pumpkin_plugin_api::Result<()> {
        CONFIG.with_borrow(|c| {
            c.write(&context.get_data_folder());
        });
        SCRIPT_RUNTIME.with(|r| {
            if let Some(ref mut runtime) = *r.borrow_mut() {
                runtime.disable_all_plugins();
            }
        });
        tracing::info!("PLua unloaded.");
        Ok(())
    }
}

pumpkin_plugin_api::register_plugin!(PLuaPlugin);
