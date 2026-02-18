# MOD-001: Extract Building Stats to Data Files

## Priority: T2 (Depth)
## Effort: Medium (3-5 days)
## Source: modding_architecture.md -- Data-Driven Architecture, master_architecture.md T2

## Description
All building parameters (capacity, cost, levels, upgrade thresholds) are hardcoded in Rust. Extract to RON/TOML data files with a BuildingDef struct. Load at startup, validate, and use as the source of truth.

## Acceptance Criteria
- [ ] `BuildingDef` struct: zone type, level, capacity, cost, maintenance, size, model
- [ ] `assets/data/buildings.ron` (or .toml) with all building definitions
- [ ] Loaded and validated at startup
- [ ] Building spawner uses data file values instead of hardcoded constants
- [ ] Invalid data files produce clear error messages
