# INFRA-010: Geologically-Aware Resource Distribution
**Priority:** T2
**Complexity:** M (1-3 days)
**Dependencies:** INFRA-001, INFRA-009
**Source:** procedural_terrain.md, Section 5.1-5.5

## Description
Replace the current hash-based resource placement in `natural_resources.rs` with geology-aware distribution. Ore deposits should appear in mountainous regions (high elevation, high slope). Fertile soil zones near river floodplains. Oil/gas in sedimentary basins (low-lying flat areas). Forest density from biome/moisture. Each resource type uses noise + elevation + moisture masks. Add resource discovery mechanic: resources are hidden until geological survey building is placed nearby.

## Definition of Done
- [ ] Ore/mineral deposits concentrated in high-elevation rocky terrain
- [ ] Fertile soil zones in river valleys and floodplains
- [ ] Oil/gas reserves in low-elevation flat areas
- [ ] Forest density driven by moisture and biome
- [ ] Resource discovery requires survey/exploration action
- [ ] Tests pass

## Test Plan
- Unit: No ore deposits spawn in water cells
- Unit: Fertile soil concentrates near rivers
- Integration: Resource distribution looks geologically plausible

## Pitfalls
- Existing resource placement uses hash(cell_position); migration needed for save compat
- All resources in one area makes some starts trivially easy; ensure distribution across map

## Relevant Code
- `crates/simulation/src/natural_resources.rs` -- current hash-based placement
- `crates/simulation/src/terrain.rs` -- elevation/moisture data
