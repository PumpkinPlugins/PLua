pub mod events;
pub mod manifest;
pub mod runtime;
pub mod structs;
pub mod worker;

use std::path::PathBuf;
use std::sync::Once;
use std::sync::mpsc::{self, Sender};
use std::thread;
use std::time::Duration;

use anyhow::{Result, anyhow};

use self::worker::{LuaCommand, run_lua_worker};

static INIT: Once = Once::new();
static mut COMMAND_SENDER: Option<Sender<LuaCommand>> = None;
static mut WORKER_HANDLE: Option<thread::JoinHandle<()>> = None;
static mut IS_INITIALIZED: bool = false;

pub fn init_lua_manager(data_dir: String) -> Result<()> {
    INIT.call_once(|| {
        let (tx, rx) = mpsc::channel::<LuaCommand>();

        unsafe {
            COMMAND_SENDER = Some(tx.clone());
        }

        let data_path = data_dir.clone();
        let tx = tx.clone();
        let handle = thread::spawn(move || {
            run_lua_worker(rx, tx, data_path);
        });

        unsafe {
            WORKER_HANDLE = Some(handle);
            IS_INITIALIZED = true;
        }
    });

    unsafe {
        if !IS_INITIALIZED {
            return Err(anyhow!(
                "Lua manager initialization failed - no valid Lua state"
            ));
        }
    }

    Ok(())
}

fn get_sender() -> Result<Sender<LuaCommand>> {
    unsafe {
        if !IS_INITIALIZED {
            return Err(anyhow!(
                "LuaManager not initialized or initialization failed"
            ));
        }

        #[allow(static_mut_refs)]
        if let Some(sender) = COMMAND_SENDER.as_ref() {
            Ok(sender.clone())
        } else {
            Err(anyhow!("LuaManager not initialized"))
        }
    }
}

pub fn reload() -> Result<()> {
    let sender = get_sender()?;

    if unsafe { !IS_INITIALIZED } {
        return Err(anyhow!(
            "Lua runtime not properly initialized - no valid Lua state"
        ));
    }

    let (tx, rx) = mpsc::channel();
    sender
        .send(LuaCommand::Reload { response: tx })
        .map_err(|_| anyhow!("Failed to send command to Lua worker"))?;

    rx.recv_timeout(Duration::from_secs(10))
        .map_err(|_| anyhow!("Lua worker disconnected or reload timed out"))?
}

pub fn get_plugin_list() -> Vec<(String, bool)> {
    let sender = match get_sender() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error getting sender: {}", e);
            return Vec::new();
        }
    };

    let (tx, rx) = mpsc::channel();
    if let Err(e) = sender.send(LuaCommand::GetPluginList { response: tx }) {
        eprintln!("Failed to send command to Lua worker: {}", e);
        return Vec::new();
    }

    match rx.recv() {
        Ok(list) => list,
        Err(e) => {
            eprintln!("Lua worker disconnected: {}", e);
            Vec::new()
        }
    }
}

pub fn enable_plugin(name: &str) -> Result<bool> {
    let sender = get_sender()?;

    if unsafe { !IS_INITIALIZED } {
        return Err(anyhow!(
            "Lua runtime not properly initialized - no valid Lua state"
        ));
    }

    let (tx, rx) = mpsc::channel();
    sender
        .send(LuaCommand::EnablePlugin {
            name: name.to_string(),
            response: tx,
        })
        .map_err(|_| anyhow!("Failed to send command to Lua worker"))?;

    let result = rx.recv_timeout(std::time::Duration::from_secs(5));
    match result {
        Ok(plugin_result) => plugin_result,
        Err(_) => Err(anyhow!(
            "Lua worker disconnected, timed out, or failed to process request"
        )),
    }
}

pub fn disable_plugin(name: &str) -> Result<bool> {
    let sender = get_sender()?;

    if unsafe { !IS_INITIALIZED } {
        return Err(anyhow!(
            "Lua runtime not properly initialized - no valid Lua state"
        ));
    }

    let (tx, rx) = mpsc::channel();
    sender
        .send(LuaCommand::DisablePlugin {
            name: name.to_string(),
            response: tx,
        })
        .map_err(|_| anyhow!("Failed to send command to Lua worker"))?;

    let result = rx.recv_timeout(std::time::Duration::from_secs(5));
    match result {
        Ok(plugin_result) => plugin_result,
        Err(_) => Err(anyhow!(
            "Lua worker disconnected, timed out, or failed to process request"
        )),
    }
}

pub fn reload_plugin(name: &str) -> Result<bool> {
    let sender = get_sender()?;

    if unsafe { !IS_INITIALIZED } {
        return Err(anyhow!(
            "Lua runtime not properly initialized - no valid Lua state"
        ));
    }

    let (tx, rx) = mpsc::channel();
    sender
        .send(LuaCommand::ReloadPlugin {
            name: name.to_string(),
            response: tx,
        })
        .map_err(|_| anyhow!("Failed to send command to Lua worker"))?;

    let result = rx.recv_timeout(std::time::Duration::from_secs(5));
    match result {
        Ok(plugin_result) => plugin_result,
        Err(_) => Err(anyhow!(
            "Lua worker disconnected, timed out, or failed to process request"
        )),
    }
}

pub fn get_plugin_info(name: &str) -> Option<(String, String, String, String, bool, PathBuf)> {
    let sender = match get_sender() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error getting sender: {}", e);
            return None;
        }
    };

    let (tx, rx) = mpsc::channel();
    if let Err(e) = sender.send(LuaCommand::GetPluginInfo {
        name: name.to_string(),
        response: tx,
    }) {
        eprintln!("Failed to send command to Lua worker: {}", e);
        return None;
    }

    match rx.recv() {
        Ok(info) => info,
        Err(e) => {
            eprintln!("Lua worker disconnected: {}", e);
            None
        }
    }
}
