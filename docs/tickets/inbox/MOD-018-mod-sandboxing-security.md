# MOD-018: Mod Sandboxing and Security

## Priority: T5 (Stretch)
## Effort: Medium (3-5 days)
## Source: modding_architecture.md -- Security

## Description
Script mods must be sandboxed: no filesystem access, no network, limited CPU/memory. Native plugins require explicit user consent. WASM provides this inherently; Lua requires custom sandbox configuration.

## Acceptance Criteria
- [ ] Lua sandbox: disable `os`, `io`, `loadfile`, `dofile` modules
- [ ] WASM: fuel metering enforced per tick
- [ ] WASM: memory limit enforced (configurable pages)
- [ ] Native plugins: security warning shown before loading
- [ ] Mod permissions system (what APIs a mod can access)
- [ ] Runaway mod detection and termination
