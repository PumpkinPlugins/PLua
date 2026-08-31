---@meta

---@class Log
---@field info fun(message: string)
---@field warn fun(message: string)
---@field error fun(message: string)
---@field debug fun(message: string)

---@class Server
---@field broadcast_message fun(message: string)

---@class Events
---@field register_listener fun(event_type: string, callback: fun(...: any)): string
---@field unregister_listener fun(event_type: string, listener_id: string): boolean

---@class Pumpkin
---@field log Log
---@field server Server
---@field events Events

---@type Pumpkin
pumpkin = nil

---@class Plugin
---@field name string
---@field description string
---@field version string
---@field author string
---@field on_enable fun()
---@field on_disable fun()
