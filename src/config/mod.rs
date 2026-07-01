use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct PLuaConfig {
    pub enabled_plugins: Vec<String>,
}

impl PLuaConfig {
    pub fn load(data_folder: &str) -> Self {
        let config_path = PathBuf::from(data_folder).join("config.json");
        if config_path.exists() {
            match fs::read_to_string(&config_path) {
                Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
                Err(_) => Self::default(),
            }
        } else {
            let config = Self::default();
            let _ = fs::create_dir_all(data_folder);
            let _ = fs::write(&config_path, serde_json::to_string_pretty(&config).unwrap_or_default());
            config
        }
    }

    pub fn write(&self, data_folder: &str) {
        let config_path = PathBuf::from(data_folder).join("config.json");
        let _ = fs::create_dir_all(data_folder);
        let _ = fs::write(&config_path, serde_json::to_string_pretty(self).unwrap_or_default());
    }

    pub fn enable_plugin(&mut self, name: &str) {
        if !self.enabled_plugins.contains(&name.to_string()) {
            self.enabled_plugins.push(name.to_string());
        }
    }

    pub fn disable_plugin(&mut self, name: &str) {
        self.enabled_plugins.retain(|n| n != name);
    }

    pub fn is_enabled(&self, name: &str) -> bool {
        self.enabled_plugins.contains(&name.to_string())
    }
}
