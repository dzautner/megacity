# MOD-011: WASM Scripting Runtime via wasmtime

## Priority: T5 (Stretch)
## Effort: Large (2-3 weeks)
## Source: modding_architecture.md -- Scripting Language Integration (WASM)

## Description
Integrate WASM modding via the `wasmtime` crate. Provides near-native performance (5-10x faster than Lua) with built-in sandboxing. Supports Rust, C, AssemblyScript, and other WASM-targeting languages.

## Acceptance Criteria
- [ ] `wasmtime` crate integrated
- [ ] `WasmModRuntime` with fuel metering and memory limits
- [ ] Host functions: city_get_population, traffic_get_density, building_set_capacity, show_notification
- [ ] Command queue pattern: WASM enqueues commands, host applies them
- [ ] GameSnapshot struct for read-only game state
- [ ] CPU budget: fuel metering prevents runaway mods
- [ ] Memory budget: configurable memory page limit
- [ ] Example mod in Rust and AssemblyScript
