use anyhow::{Context as AnyhowContext, Result};
use mlua::Lua;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::ConfigManager;
use crate::lua::manifest::LuaPluginManifest;
use crate::lua::{events, structs};

pub struct LuaPlugin {
    pub manifest: LuaPluginManifest,
    pub file_path: PathBuf,
    pub enabled: bool,
}

pub struct LuaRuntime {
    pub lua: Lua,
    pub plugins_dir: PathBuf,
    pub plugins: HashMap<String, LuaPlugin>,
}

impl LuaRuntime {
    pub fn new(data_dir: &Path) -> Result<Self> {
        let plugins_dir = data_dir.join("plugins");
        fs::create_dir_all(&plugins_dir).context("Failed to create plugins directory")?;

        let lua = Lua::new();
        lua.sandbox(true)?;

        Ok(Self {
            lua,
            plugins_dir,
            plugins: HashMap::new(),
        })
    }

    pub fn discover_plugins(&mut self) -> Result<()> {
        self.plugins.clear();

        let entries =
            fs::read_dir(&self.plugins_dir).context("Failed to read plugins directory")?;

        for entry in entries {
            let entry = entry.context("Failed to read directory entry")?;
            let path = entry.path();

            if path.is_file() && path.extension().is_some_and(|ext| ext == "lua") {
                self.load_plugin_metadata(&path)?;
            }
        }

        Ok(())
    }

    fn load_plugin_metadata(&mut self, path: &Path) -> Result<()> {
        let script = fs::read_to_string(path)
            .with_context(|| format!("Failed to read plugin file: {:?}", path))?;

        // Only used for metadata extraction
        let manifest = self
            .lua
            .load(&script)
            .set_name(path.file_name().unwrap().to_string_lossy().as_ref())
            .eval::<LuaPluginManifest>()?;

        let plugin = LuaPlugin {
            manifest,
            file_path: path.to_path_buf(),
            enabled: false,
        };

        self.plugins.insert(plugin.manifest.name.clone(), plugin);

        Ok(())
    }

    pub fn init_api(&self) -> Result<()> {
        let lua = &self.lua;

        let pumpkin_table = lua.create_table()?;
        pumpkin_table.set("log", structs::Log)?;
        pumpkin_table.set("server", structs::Server)?;

        {
            let events_table = lua.create_table()?;
            events_table.set("", structs::Events)?;
            events::player_join::setup_lua_event(lua, &events_table)?;
            events::player_leave::setup_lua_event(lua, &events_table)?;
            events::player_chat::setup_lua_event(lua, &events_table)?;
            events::block_place::setup_lua_event(lua, &events_table)?;
            events::block_break::setup_lua_event(lua, &events_table)?;
            pumpkin_table.set("events", events_table)?;
        }

        lua.globals().set("pumpkin", pumpkin_table.clone())?;
        Ok(())
    }

    pub fn enable_plugin(&mut self, name: &str) -> Result<bool> {
        if let Some(plugin) = self.plugins.get_mut(name) {
            if plugin.enabled {
                return Ok(false);
            }

            let script = fs::read_to_string(&plugin.file_path)
                .with_context(|| format!("Failed to read plugin file: {:?}", plugin.file_path))?;

            self.lua
                .load(&script)
                .set_name(
                    plugin
                        .file_path
                        .file_name()
                        .unwrap()
                        .to_string_lossy()
                        .as_ref(),
                )
                .exec()
                .with_context(|| {
                    format!("Failed to execute plugin script: {:?}", plugin.file_path)
                })?;

            if let Some(on_enable) = &plugin.manifest.on_enable {
                on_enable
                    .call::<()>(())
                    .with_context(|| format!("Failed to call on_enable for plugin {}", name))?;
            }

            plugin.enabled = true;
            Ok(true)
        } else {
            log::warn!("Attempted to enable unknown plugin: {}", name);
            Ok(false)
        }
    }

    pub fn disable_plugin(&mut self, name: &str) -> Result<bool> {
        if let Some(plugin) = self.plugins.get_mut(name) {
            if !plugin.enabled {
                return Ok(false);
            }

            if let Some(on_disable) = &plugin.manifest.on_disable {
                on_disable
                    .call::<()>(())
                    .with_context(|| format!("Failed to call on_disable for plugin {}", name))?;
            }

            plugin.enabled = false;
            Ok(true)
        } else {
            log::warn!("Attempted to disable unknown plugin: {}", name);
            Ok(false)
        }
    }

    pub fn load_enabled_plugins(&mut self, config_manager: &ConfigManager) -> Result<()> {
        for plugin_name in &config_manager.config.enabled_plugins {
            if let Some(plugin) = self.plugins.get(plugin_name) {
                if !plugin.enabled {
                    if let Err(e) = self.enable_plugin(plugin_name) {
                        log::error!("Failed to enable plugin {}: {}", plugin_name, e);
                    }
                }
            } else {
                log::warn!("Enabled plugin {} not found", plugin_name);
            }
        }

        Ok(())
    }

    pub fn disable_all_plugins(&mut self) -> Result<()> {
        let mut to_disable = vec![];

        for (name, plugin) in &mut self.plugins {
            if plugin.enabled {
                to_disable.push(name.clone());
            }
        }

        for name in to_disable {
            if let Err(e) = self.disable_plugin(name.as_str()) {
                log::error!("Failed to disable plugin {}: {}", name, e);
            }
        }

        Ok(())
    }

    pub fn reload_plugin(&mut self, name: &str) -> Result<bool> {
        let was_enabled = if let Some(plugin) = self.plugins.get(name) {
            plugin.enabled
        } else {
            return Ok(false);
        };

        if was_enabled {
            self.disable_plugin(name)?;
        }

        let fp = {
            let plugin = self.plugins.get(name);
            if plugin.is_none() {
                return Ok(false);
            }
            plugin.unwrap().file_path.clone()
        };

        self.load_plugin_metadata(&fp)?;

        if was_enabled {
            self.enable_plugin(name)?;
        }

        Ok(true)
    }
}
