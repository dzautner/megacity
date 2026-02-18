# MOD-008: Data Override Hierarchy (Base -> Mod -> User)

## Priority: T5 (Stretch)
## Effort: Medium (3-5 days)
## Source: modding_architecture.md -- Override Hierarchy

## Description
Implement a three-tier data override system: base game data -> mod data -> user data. Later layers override earlier ones. Supports partial overrides (only override specific fields).

## Acceptance Criteria
- [ ] Base game data loaded first from `assets/data/`
- [ ] Mod data loaded from `mods/{mod_id}/data/` and merged
- [ ] User data loaded from `user/data/` (highest priority)
- [ ] Partial override: mod file can override just `cost` without specifying all fields
- [ ] Override conflicts logged
