-- Hello World Example Plugin for PLua

return {
    name = "HelloWorld",
    description = "A simple hello world plugin for PLua",
    version = "1.0.0",
    author = "PLua Team",

    on_enable = function()
        pumpkin.log.info("Hello World plugin enabled!")
        pumpkin.server.broadcast_message("Hello, world! PLua is running.")
    end,

    on_disable = function()
        pumpkin.log.info("Hello World plugin disabled!")
    end,
}
