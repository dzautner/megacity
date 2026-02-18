# CSLN-007: Modding API Stability Foundation
**Priority:** T5
**Complexity:** XL
**Dependencies:** UI-003
**Source:** cities_skylines_analysis.md, section 16.7 (lesson 4), 17; master_architecture.md, section T5

## Description
CS1's modding ecosystem (700K+ items) was its greatest asset. CS2's modding failure was existential. Megacity must plan for modding from early architecture. This ticket establishes the foundation.

- Stable public API for game data structures (building, citizen, road)
- Plugin interface: WASM or native DLL loading
- Asset pipeline: import custom building meshes, textures
- Data-driven parameters (UI-003) as first modding layer
- Mod load order and conflict detection
- Sandbox: mods cannot access filesystem directly

## Definition of Done
- [ ] Stable public API documented
- [ ] At least one proof-of-concept mod loads and modifies game behavior
- [ ] Custom building asset loadable
- [ ] Mod manager UI with enable/disable

## Test Plan
- Integration: Load test mod that adds a building type
- Integration: Load two conflicting mods, verify conflict detection

## Pitfalls
- API stability requires discipline (no breaking changes after API freeze)
- WASM sandboxing limits what mods can do (tradeoff with power)
- Bevy's rapidly evolving API makes stable wrapping difficult

## Relevant Code
- All public-facing data structures -- need stable serialization
- `crates/app/src/main.rs` -- plugin loading
