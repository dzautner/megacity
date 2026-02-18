# INFRA-107: Scripting Integration (Lua or WASM)
**Priority:** T5
**Complexity:** XL (1-2 weeks)
**Dependencies:** INFRA-106
**Source:** master_architecture.md, M6

## Description
Add sandboxed scripting support using Lua or WASM for safe mod execution. Scripts can define custom events, policies, building behaviors, and scenario logic. Sandboxed execution prevents filesystem/network access. API bindings for game state queries and mutations. Script hot-reload for rapid mod development.

## Definition of Done
- [ ] Scripting runtime embedded (Lua or WASM)
- [ ] Sandboxed execution (no filesystem/network access)
- [ ] Game state API bindings (read population, budget, etc.)
- [ ] Mutation API (place building, adjust policy, etc.)
- [ ] Hot-reload for scripts
- [ ] Script error handling (no crashes from bad scripts)
- [ ] Tests pass

## Test Plan
- Unit: Script can query population count
- Unit: Malicious script cannot access filesystem
- Integration: Custom scenario defined entirely in script

## Pitfalls
- Lua is simpler but less safe; WASM is more secure but harder to write
- API surface is large; start with read-only state access
- Performance: scripts running every tick must be fast

## Relevant Code
- New module in `crates/modding/`
