use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use piccolo::{
    Callback, CallbackReturn, Closure, Executor, Lua, RuntimeError, StaticError, Table, Value,
};
use pumpkin_plugin_api::Server;
use tracing;

use crate::config::PLuaConfig;
use crate::events::ALL_EVENT_TYPES;
use crate::script::manifest::PluginManifest;

static mut LUA: *mut Lua = std::ptr::null_mut();

pub fn get_lua() -> Option<&'static mut Lua> {
    // unsafe: WASM is single-threaded
    unsafe { if LUA.is_null() { None } else { Some(&mut *LUA) } }
}

fn runtime_err(msg: impl Into<String>) -> StaticError {
    StaticError::Runtime(RuntimeError::from(anyhow::anyhow!("{}", msg.into())))
}

fn get_or_create_storage(ctx: piccolo::Context<'_>) -> Table<'_> {
    let storage = ctx.globals().get(ctx, "__plua_storage");
    if let Value::Table(t) = storage {
        return t;
    }
    let t = Table::new(&ctx);
    ctx.globals().set(ctx, "__plua_storage", t).ok();
    t
}

#[derive(Debug, Clone)]
pub struct PluginInfo {
    pub manifest: PluginManifest,
    pub file_path: PathBuf,
    pub enabled: bool,
}

#[derive(Debug)]
pub struct PluginRuntime {
    plugins_dir: PathBuf,
    plugins: HashMap<String, PluginInfo>,
}

impl PluginRuntime {
    pub fn new(data_folder: &str) -> Self {
        let plugins_dir = PathBuf::from(data_folder).join("plugins");
        let _ = fs::create_dir_all(&plugins_dir);

        let lua = Lua::core();

        // unsafe: WASM is single-threaded
        unsafe {
            LUA = Box::into_raw(Box::new(lua));
        }

        PluginRuntime {
            plugins_dir,
            plugins: HashMap::new(),
        }
    }

    pub fn init_api(&mut self, server: Server) {
        let lua = get_lua().expect("Lua not initialized");

        let _ = lua.try_enter(|ctx| {
            let pumpkin = Table::new(&ctx);

            let log = Table::new(&ctx);
            log.set(
                ctx,
                "info",
                Callback::from_fn(&ctx, |ctx, _, mut stack| {
                    let (msg,): (String,) = stack.consume(ctx).unwrap_or_default();
                    tracing::info!("[Lua] {}", msg);
                    stack.replace(ctx, ());
                    Ok(CallbackReturn::Return)
                }),
            )
            .ok();
            log.set(
                ctx,
                "warn",
                Callback::from_fn(&ctx, |ctx, _, mut stack| {
                    let (msg,): (String,) = stack.consume(ctx).unwrap_or_default();
                    tracing::warn!("[Lua] {}", msg);
                    stack.replace(ctx, ());
                    Ok(CallbackReturn::Return)
                }),
            )
            .ok();
            log.set(
                ctx,
                "error",
                Callback::from_fn(&ctx, |ctx, _, mut stack| {
                    let (msg,): (String,) = stack.consume(ctx).unwrap_or_default();
                    tracing::error!("[Lua] {}", msg);
                    stack.replace(ctx, ());
                    Ok(CallbackReturn::Return)
                }),
            )
            .ok();
            log.set(
                ctx,
                "debug",
                Callback::from_fn(&ctx, |ctx, _, mut stack| {
                    let (msg,): (String,) = stack.consume(ctx).unwrap_or_default();
                    tracing::debug!("[Lua] {}", msg);
                    stack.replace(ctx, ());
                    Ok(CallbackReturn::Return)
                }),
            )
            .ok();
            pumpkin.set(ctx, "log", log).ok();

            let server_table = Table::new(&ctx);
            let broadcast_func = Callback::from_fn(&ctx, move |ctx, _, mut stack| {
                let (msg,): (String,) = stack.consume(ctx).unwrap_or_default();
                server.broadcast(&msg);
                stack.replace(ctx, ());
                Ok(CallbackReturn::Return)
            });
            server_table
                .set(ctx, "broadcast_message", broadcast_func)
                .ok();
            pumpkin.set(ctx, "server", server_table).ok();

            let events_table = Table::new(&ctx);
            for event_type in ALL_EVENT_TYPES {
                events_table.set(ctx, *event_type, Table::new(&ctx)).ok();
            }

            let register_listener = Callback::from_fn(&ctx, move |ctx, _, mut stack| {
                let (event_type, callback): (String, Value) =
                    stack.consume(ctx).unwrap_or_default();

                tracing::info!(
                    "PLua: register_listener called for '{}', callback is nil: {}",
                    event_type,
                    callback.is_nil()
                );

                let listener_id = format!(
                    "{}_{}",
                    event_type,
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_millis())
                        .unwrap_or(0),
                );

                let event_key = piccolo::String::from_slice(&ctx, event_type.as_bytes());
                let id_key = piccolo::String::from_slice(&ctx, listener_id.as_bytes());

                let pumpkin_val = ctx.globals().get(ctx, "pumpkin");
                let found_pumpkin = matches!(pumpkin_val, Value::Table(_));
                tracing::info!(
                    "PLua: pumpkin found: {}, event_key: {}",
                    found_pumpkin,
                    event_type
                );

                if let Value::Table(p) = pumpkin_val {
                    let events_val = p.get(ctx, "events");
                    let found_events = matches!(events_val, Value::Table(_));
                    tracing::debug!("PLua: events found: {}", found_events);

                    if let Value::Table(events) = events_val {
                        let listeners_val = events.get(ctx, event_key);
                        let found_listeners = matches!(listeners_val, Value::Table(_));
                        tracing::debug!("PLua: listeners found: {}", found_listeners);

                        if let Value::Table(listeners) = listeners_val {
                            listeners.set(ctx, id_key, callback).ok();
                            tracing::info!(
                                "PLua: stored listener '{}' for event '{}'",
                                listener_id,
                                event_type
                            );
                        }
                    }
                }

                let result = piccolo::String::from_slice(&ctx, listener_id.as_bytes());
                stack.replace(ctx, result);
                Ok(CallbackReturn::Return)
            });
            events_table
                .set(ctx, "register_listener", register_listener)
                .ok();

            let unregister_listener = Callback::from_fn(&ctx, |ctx, _, mut stack| {
                let (event_type, listener_id): (String, String) =
                    stack.consume(ctx).unwrap_or_default();

                let event_key = piccolo::String::from_slice(&ctx, event_type.as_bytes());
                let id_key = piccolo::String::from_slice(&ctx, listener_id.as_bytes());

                let pumpkin_val = ctx.globals().get(ctx, "pumpkin");
                if let Value::Table(p) = pumpkin_val {
                    let events_val = p.get(ctx, "events");
                    if let Value::Table(events) = events_val {
                        let listeners_val = events.get(ctx, event_key);
                        if let Value::Table(listeners) = listeners_val {
                            listeners.set(ctx, id_key, Value::Nil).ok();
                        }
                    }
                }

                stack.replace(ctx, ());
                Ok(CallbackReturn::Return)
            });
            events_table
                .set(ctx, "unregister_listener", unregister_listener)
                .ok();

            pumpkin.set(ctx, "events", events_table).ok();
            ctx.set_global("pumpkin", pumpkin).ok();

            Ok(())
        });
    }

    fn load_manifest(&self, path: &Path) -> Result<PluginManifest, String> {
        let file_name = path
            .file_stem()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        Ok(PluginManifest {
            name: file_name.to_string(),
            description: String::new(),
            version: "1.0.0".into(),
            author: "Unknown".into(),
            on_enable_ref: None,
            on_disable_ref: None,
        })
    }

    pub fn discover_plugins(&mut self) {
        if !self.plugins_dir.exists() {
            return;
        }
        let entries = match fs::read_dir(&self.plugins_dir) {
            Ok(e) => e,
            Err(_) => return,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path
                .extension()
                .is_some_and(|ext| ext == "lua" || ext == "luau")
            {
                let manifest = match self.load_manifest(&path) {
                    Ok(m) => m,
                    Err(e) => {
                        tracing::warn!(
                            "Failed to load manifest for {:?}: {}",
                            path.file_name().unwrap_or_default(),
                            e
                        );
                        continue;
                    }
                };
                let name = manifest.name.clone();
                tracing::info!("Discovered plugin: {}", name);
                self.plugins.insert(
                    name,
                    PluginInfo {
                        manifest,
                        file_path: path,
                        enabled: false,
                    },
                );
            }
        }
    }

    pub fn enable_plugin(&mut self, name: &str) -> Result<(), String> {
        match self.plugins.get(name) {
            Some(plugin) if plugin.enabled => {
                return Err(format!("Plugin '{}' is already enabled", name));
            }
            Some(_) => {}
            None => return Err(format!("Plugin '{}' not found", name)),
        }

        let file_path = self.plugins[name].file_path.clone();
        let content =
            fs::read_to_string(&file_path).map_err(|e| format!("Failed to read plugin: {}", e))?;

        let lua = get_lua().ok_or("Lua not initialized")?;
        let file_name = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        let name_clone = name.to_string();
        let disable_key_str = format!("__plua_disable_{}", name);

        let exec = lua
            .try_enter(|ctx| {
                let closure = Closure::load(ctx, Some(file_name), content.as_bytes())
                    .map_err(|e| runtime_err(format!("Compile error: {:?}", e)))?;
                Ok(ctx.stash(Executor::start(ctx, closure.into(), ())))
            })
            .map_err(|e: StaticError| format!("Failed to load plugin '{}': {}", name, e))?;

        lua.finish(&exec);

        let on_enable_exec = lua
            .try_enter(|ctx| {
                let executor = ctx.fetch(&exec);
                match executor.take_result::<Value>(ctx) {
                    Ok(Ok(Value::Table(table))) => {
                        let disable_key =
                            piccolo::String::from_slice(&ctx, disable_key_str.as_bytes());
                        if let Value::Function(on_disable) = table.get(ctx, "on_disable") {
                            let storage = get_or_create_storage(ctx);
                            storage.set(ctx, disable_key, on_disable).ok();
                        }
                        if let Value::Function(on_enable) = table.get(ctx, "on_enable") {
                            return Ok(Some(ctx.stash(Executor::start(ctx, on_enable, ()))));
                        }
                        Ok(None)
                    }
                    _ => Ok(None),
                }
            })
            .map_err(|e: StaticError| format!("Failed to extract plugin '{}': {}", name, e))?;

        if let Some(enable_exec) = on_enable_exec {
            let _ = lua.execute::<()>(&enable_exec);
        }

        tracing::info!("Plugin '{}' enabled.", name);

        let mut plugin_info = self.plugins.remove(&name_clone).unwrap();
        plugin_info.enabled = true;
        self.plugins.insert(name_clone, plugin_info);

        Ok(())
    }

    pub fn disable_plugin(&mut self, name: &str) -> Result<(), String> {
        match self.plugins.get(name) {
            Some(plugin) if !plugin.enabled => {
                return Err(format!("Plugin '{}' is not enabled", name));
            }
            Some(_) => {}
            None => return Err(format!("Plugin '{}' not found", name)),
        }

        let lua = get_lua().ok_or("Lua not initialized")?;
        let disable_key_str = format!("__plua_disable_{}", name);

        let on_disable_exec = lua
            .try_enter(|ctx| {
                let storage = get_or_create_storage(ctx);
                let disable_key = piccolo::String::from_slice(&ctx, disable_key_str.as_bytes());
                let val = storage.get(ctx, disable_key);
                if let Value::Function(func) = val {
                    storage.set(ctx, disable_key, Value::Nil).ok();
                    Ok(Some(ctx.stash(Executor::start(ctx, func, ()))))
                } else {
                    Ok(None)
                }
            })
            .map_err(|e: StaticError| format!("Failed to call on_disable: {}", e))?;

        if let Some(disable_exec) = on_disable_exec {
            let _ = lua.execute::<()>(&disable_exec);
        }

        if let Some(plugin) = self.plugins.get_mut(name) {
            plugin.enabled = false;
        }

        tracing::info!("Plugin '{}' disabled.", name);

        Ok(())
    }

    pub fn load_enabled_plugins(&mut self, config: &PLuaConfig) {
        let names: Vec<String> = config.enabled_plugins.clone();
        for name in names {
            if let Err(e) = self.enable_plugin(&name) {
                tracing::error!("Failed to enable plugin '{}': {}", name, e);
            }
        }
    }

    pub fn disable_all_plugins(&mut self) {
        let names: Vec<String> = self.plugins.keys().cloned().collect();
        for name in names {
            let _ = self.disable_plugin(&name);
        }
    }

    pub fn reload_plugin(&mut self, name: &str) -> Result<(), String> {
        let was_enabled = self.plugins.get(name).map(|p| p.enabled).unwrap_or(false);
        if was_enabled {
            self.disable_plugin(name)?;
        }
        if was_enabled {
            self.enable_plugin(name)?;
        }
        tracing::info!("Plugin '{}' reloaded.", name);
        Ok(())
    }

    pub fn get_plugin_list(&self) -> Vec<(String, bool, String, String)> {
        self.plugins
            .iter()
            .map(|(name, info)| {
                (
                    name.clone(),
                    info.enabled,
                    info.manifest.version.clone(),
                    info.manifest.description.clone(),
                )
            })
            .collect()
    }

    pub fn get_plugin_info(&self, name: &str) -> Option<(&PluginManifest, &PathBuf, bool)> {
        self.plugins
            .get(name)
            .map(|info| (&info.manifest, &info.file_path, info.enabled))
    }

    pub fn plugin_exists(&self, name: &str) -> bool {
        self.plugins.contains_key(name)
    }
}

impl Drop for PluginRuntime {
    fn drop(&mut self) {
        // unsafe: WASM is single-threaded
        unsafe {
            if !LUA.is_null() {
                drop(Box::from_raw(LUA));
                LUA = std::ptr::null_mut();
            }
        }
    }
}
