# INFRA-106: Full Modding SDK (Native Plugins)
**Priority:** T5
**Complexity:** XL (1-2 weeks)
**Dependencies:** INFRA-077
**Source:** master_architecture.md, M6

## Description
Create a stable modding API with native plugin support. Mods can add new building types, road types, service buildings, production chains, and visual assets. Plugin interface with versioned ABI. Hot-reload in dev mode. Mod loading with dependency resolution. Mod isolation (mods cannot corrupt core game state).

## Definition of Done
- [ ] Stable plugin API with version guarantees
- [ ] Mod loading and initialization
- [ ] Hot-reload support in dev builds
- [ ] Dependency resolution between mods
- [ ] Example mod demonstrating API usage
- [ ] Mod API documentation
- [ ] Tests pass

## Test Plan
- Unit: Plugin loads and registers new building type
- Unit: Incompatible plugin version rejected gracefully
- Integration: Example mod works in-game

## Pitfalls
- ABI stability across Rust versions is very difficult
- Hot-reload with dynamic linking requires careful memory management
- Mod security: untrusted native code is dangerous

## Relevant Code
- New crate: `crates/modding/`
- `crates/app/src/main.rs` -- mod loading at startup
