use pumpkin_plugin_api::{
    command::{Command, CommandError, CommandNode},
    command_wit::{Arg, ArgumentType, StringType},
    commands::CommandHandler,
    common::NamedColor,
    permission::{Permission, PermissionDefault, PermissionLevel},
    text::TextComponent,
};

use crate::{CONFIG, SCRIPT_RUNTIME};

const ARG_PLUGIN_NAME: &str = "plugin_name";
const PERMISSION_NODE: &str = "plua:command.plua";

struct ListPluginsHandler;

impl CommandHandler for ListPluginsHandler {
    fn handle(
        &self,
        sender: pumpkin_plugin_api::command::CommandSender,
        _server: pumpkin_plugin_api::Server,
        _args: pumpkin_plugin_api::command::ConsumedArgs,
    ) -> pumpkin_plugin_api::Result<i32, CommandError> {
        let plugin_list = SCRIPT_RUNTIME.with(|rt| {
            if let Some(ref runtime) = *rt.borrow() {
                runtime.get_plugin_list()
            } else {
                Vec::new()
            }
        });

        if plugin_list.is_empty() {
            let msg = TextComponent::text("No Lua plugins found.").color_named(NamedColor::Yellow);
            sender.send_message(msg);
            return Ok(0);
        }

        let header = TextComponent::text("=== Lua Plugins ===").color_named(NamedColor::Gold);
        sender.send_message(header);

        for (name, enabled, _version, _desc) in plugin_list {
            let status_color = if enabled {
                NamedColor::Green
            } else {
                NamedColor::Red
            };
            let status_text = if enabled { "Enabled" } else { "Disabled" };

            let line = TextComponent::text(&format!("- {name} ["))
                .add_child(TextComponent::text(status_text).color_named(status_color))
                .add_child(TextComponent::text("]"));
            sender.send_message(line);
        }

        Ok(0)
    }
}

struct EnablePluginHandler;

impl CommandHandler for EnablePluginHandler {
    fn handle(
        &self,
        sender: pumpkin_plugin_api::command::CommandSender,
        _server: pumpkin_plugin_api::Server,
        args: pumpkin_plugin_api::command::ConsumedArgs,
    ) -> pumpkin_plugin_api::Result<i32, CommandError> {
        let Arg::Simple(plugin_name) = args.get_value(ARG_PLUGIN_NAME) else {
            return Err(CommandError::InvalidConsumption(Some(
                ARG_PLUGIN_NAME.into(),
            )));
        };

        let result = SCRIPT_RUNTIME.with(|rt| {
            (*rt.borrow_mut())
                .as_mut()
                .map(|runtime| runtime.enable_plugin(&plugin_name))
        });

        match result {
            Some(Ok(())) => {
                CONFIG.with(|c| c.borrow_mut().enable_plugin(&plugin_name));
                let msg = TextComponent::text(&format!("Plugin '{plugin_name}' ")).add_child(
                    TextComponent::text("has been enabled.").color_named(NamedColor::Green),
                );
                sender.send_message(msg);
                Ok(0)
            }
            Some(Err(e)) => {
                let msg =
                    TextComponent::text(&format!("Failed to enable plugin '{plugin_name}': "))
                        .add_child(TextComponent::text(&e).color_named(NamedColor::Red));
                sender.send_message(msg);
                Ok(0)
            }
            None => {
                let msg =
                    TextComponent::text("Runtime not initialized").color_named(NamedColor::Red);
                sender.send_message(msg);
                Ok(0)
            }
        }
    }
}

struct DisablePluginHandler;

impl CommandHandler for DisablePluginHandler {
    fn handle(
        &self,
        sender: pumpkin_plugin_api::command::CommandSender,
        _server: pumpkin_plugin_api::Server,
        args: pumpkin_plugin_api::command::ConsumedArgs,
    ) -> pumpkin_plugin_api::Result<i32, CommandError> {
        let Arg::Simple(plugin_name) = args.get_value(ARG_PLUGIN_NAME) else {
            return Err(CommandError::InvalidConsumption(Some(
                ARG_PLUGIN_NAME.into(),
            )));
        };

        let result = SCRIPT_RUNTIME.with(|rt| {
            (*rt.borrow_mut())
                .as_mut()
                .map(|runtime| runtime.disable_plugin(&plugin_name))
        });

        match result {
            Some(Ok(())) => {
                CONFIG.with(|c| c.borrow_mut().disable_plugin(&plugin_name));
                let msg = TextComponent::text(&format!("Plugin '{plugin_name}' ")).add_child(
                    TextComponent::text("has been disabled.").color_named(NamedColor::Green),
                );
                sender.send_message(msg);
                Ok(0)
            }
            Some(Err(e)) => {
                let msg =
                    TextComponent::text(&format!("Failed to disable plugin '{plugin_name}': "))
                        .add_child(TextComponent::text(&e).color_named(NamedColor::Red));
                sender.send_message(msg);
                Ok(0)
            }
            None => {
                let msg =
                    TextComponent::text("Runtime not initialized").color_named(NamedColor::Red);
                sender.send_message(msg);
                Ok(0)
            }
        }
    }
}

struct ReloadAllHandler;

impl CommandHandler for ReloadAllHandler {
    fn handle(
        &self,
        sender: pumpkin_plugin_api::command::CommandSender,
        _server: pumpkin_plugin_api::Server,
        _args: pumpkin_plugin_api::command::ConsumedArgs,
    ) -> pumpkin_plugin_api::Result<i32, CommandError> {
        let result = SCRIPT_RUNTIME.with(|rt| {
            if let Some(ref mut runtime) = *rt.borrow_mut() {
                let names: Vec<String> = runtime
                    .get_plugin_list()
                    .iter()
                    .map(|(n, _, _, _)| n.clone())
                    .collect();
                for name in &names {
                    let _ = runtime.reload_plugin(name);
                }
                Ok::<(), String>(())
            } else {
                Err("Runtime not initialized".into())
            }
        });

        match result {
            Ok(()) => {
                let msg = TextComponent::text("All Lua plugins have been reloaded.")
                    .color_named(NamedColor::Green);
                sender.send_message(msg);
                Ok(0)
            }
            Err(e) => {
                let msg = TextComponent::text("Failed to reload plugins: ")
                    .add_text(&e)
                    .color_named(NamedColor::Red);
                sender.send_message(msg);
                Ok(0)
            }
        }
    }
}

struct ReloadPluginHandler;

impl CommandHandler for ReloadPluginHandler {
    fn handle(
        &self,
        sender: pumpkin_plugin_api::command::CommandSender,
        _server: pumpkin_plugin_api::Server,
        args: pumpkin_plugin_api::command::ConsumedArgs,
    ) -> pumpkin_plugin_api::Result<i32, CommandError> {
        let Arg::Simple(plugin_name) = args.get_value(ARG_PLUGIN_NAME) else {
            return Err(CommandError::InvalidConsumption(Some(
                ARG_PLUGIN_NAME.into(),
            )));
        };

        let result = SCRIPT_RUNTIME.with(|rt| {
            (*rt.borrow_mut())
                .as_mut()
                .map(|runtime| runtime.reload_plugin(&plugin_name))
        });

        match result {
            Some(Ok(())) => {
                let msg = TextComponent::text(&format!("Plugin '{plugin_name}' ")).add_child(
                    TextComponent::text("has been reloaded.").color_named(NamedColor::Green),
                );
                sender.send_message(msg);
                Ok(0)
            }
            Some(Err(e)) => {
                let msg =
                    TextComponent::text(&format!("Failed to reload plugin '{plugin_name}': "))
                        .add_child(TextComponent::text(&e).color_named(NamedColor::Red));
                sender.send_message(msg);
                Ok(0)
            }
            None => {
                let msg =
                    TextComponent::text("Runtime not initialized").color_named(NamedColor::Red);
                sender.send_message(msg);
                Ok(0)
            }
        }
    }
}

struct PluginInfoHandler;

impl CommandHandler for PluginInfoHandler {
    fn handle(
        &self,
        sender: pumpkin_plugin_api::command::CommandSender,
        _server: pumpkin_plugin_api::Server,
        args: pumpkin_plugin_api::command::ConsumedArgs,
    ) -> pumpkin_plugin_api::Result<i32, CommandError> {
        let Arg::Simple(plugin_name) = args.get_value(ARG_PLUGIN_NAME) else {
            return Err(CommandError::InvalidConsumption(Some(
                ARG_PLUGIN_NAME.into(),
            )));
        };

        let info = SCRIPT_RUNTIME.with(|rt| {
            if let Some(ref runtime) = *rt.borrow() {
                runtime.get_plugin_info(&plugin_name).map(|(m, p, e)| {
                    (
                        m.name.clone(),
                        m.description.clone(),
                        m.version.clone(),
                        m.author.clone(),
                        e,
                        p.to_string_lossy().into_owned(),
                    )
                })
            } else {
                None
            }
        });

        if let Some((name, description, version, author, enabled, file_path)) = info {
            let header = TextComponent::text("=== ")
                .color_named(NamedColor::Gold)
                .add_text(&name)
                .add_child(TextComponent::text(" ===").color_named(NamedColor::Gold));
            sender.send_message(header);

            let desc = TextComponent::text("Description: ")
                .color_named(NamedColor::Yellow)
                .add_text(&description);
            sender.send_message(desc);

            let ver = TextComponent::text("Version: ")
                .color_named(NamedColor::Yellow)
                .add_text(&version);
            sender.send_message(ver);

            let auth = TextComponent::text("Author: ")
                .color_named(NamedColor::Yellow)
                .add_text(&author);
            sender.send_message(auth);

            let status_color = if enabled {
                NamedColor::Green
            } else {
                NamedColor::Red
            };
            let status_text = if enabled { "Enabled" } else { "Disabled" };
            let st = TextComponent::text("Status: ")
                .color_named(NamedColor::Yellow)
                .add_text(status_text)
                .color_named(status_color);
            sender.send_message(st);

            let file = TextComponent::text("File: ")
                .color_named(NamedColor::Yellow)
                .add_text(&file_path);
            sender.send_message(file);
        } else {
            let msg = TextComponent::text(&format!("Plugin '{plugin_name}' "))
                .add_child(TextComponent::text("not found.").color_named(NamedColor::Red));
            sender.send_message(msg);
        }

        Ok(0)
    }
}

pub fn register(context: &pumpkin_plugin_api::Context) -> pumpkin_plugin_api::Result<()> {
    context.register_permission(&Permission {
        node: PERMISSION_NODE.into(),
        description: "Allows managing Lua plugins via the /plua command".into(),
        default: PermissionDefault::Op(PermissionLevel::Four),
        children: Vec::new(),
    })?;

    let plua_cmd = Command::new(
        &["plua".to_string()],
        "Manage Lua plugins for the Pumpkin server",
    )
    .then(CommandNode::literal("list").execute(ListPluginsHandler))
    .then(
        CommandNode::literal("enable").then(
            CommandNode::argument(
                ARG_PLUGIN_NAME,
                &ArgumentType::String(StringType::SingleWord),
            )
            .execute(EnablePluginHandler),
        ),
    )
    .then(
        CommandNode::literal("disable").then(
            CommandNode::argument(
                ARG_PLUGIN_NAME,
                &ArgumentType::String(StringType::SingleWord),
            )
            .execute(DisablePluginHandler),
        ),
    )
    .then(
        CommandNode::literal("reload")
            .execute(ReloadAllHandler)
            .then(
                CommandNode::argument(
                    ARG_PLUGIN_NAME,
                    &ArgumentType::String(StringType::SingleWord),
                )
                .execute(ReloadPluginHandler),
            ),
    )
    .then(
        CommandNode::literal("info").then(
            CommandNode::argument(
                ARG_PLUGIN_NAME,
                &ArgumentType::String(StringType::SingleWord),
            )
            .execute(PluginInfoHandler),
        ),
    );

    context.register_command(plua_cmd, PERMISSION_NODE);

    Ok(())
}
