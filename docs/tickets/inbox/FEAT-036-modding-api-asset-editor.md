# FEAT-036: Modding API and Asset Editor

**Category:** Feature / Modding
**Priority:** T4
**Source:** community_wishlists.md -- Section 15 (EXTREMELY HIGH frequency)

## Summary

Stable asset editor for custom buildings, vehicles, props. Code modding API with deep access. Workshop integration. Backward compatibility across game updates. Mod manager with dependency resolution.

## Details

- Data-driven architecture: extract hardcoded values to data files (RON/JSON)
- Scripting language integration (Lua/Rhai/WASM)
- Custom building/vehicle/prop asset pipeline with validation
- Override hierarchy: base game -> mod -> user
- Workshop integration for mod distribution
- Mod manager UI with conflict detection
- Stable API versioning for backward compatibility

## Acceptance Criteria

- [ ] Asset editor for custom buildings
- [ ] Scripting API for gameplay mods
- [ ] Workshop integration
- [ ] Mods survive game updates
