# MOD-010: Lua Scripting Runtime via mlua

## Priority: T5 (Stretch)
## Effort: Large (2-3 weeks)
## Source: modding_architecture.md -- Scripting Language Integration (Lua)

## Description
Integrate Lua scripting via the `mlua` crate. Register city, buildings, traffic, citizen, UI, and event APIs. Lua mods can respond to game events, modify building capacity, show notifications, etc.

## Acceptance Criteria
- [ ] `mlua` crate integrated
- [ ] `LuaModRuntime` struct with sandbox configuration
- [ ] City API: get_population, get_treasury, get_hour, get_day
- [ ] Building API: get_at, set_capacity, spawn
- [ ] Traffic API: get_density, get_congestion
- [ ] Citizen API: count, get_average_happiness
- [ ] UI API: show_notification, add_toolbar_button
- [ ] Event API: on(event, callback), emit(event, data)
- [ ] Sandbox: no file/network access
- [ ] Performance: <1ms per tick for typical mods
