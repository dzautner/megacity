# BLDG-007: Building Age and Depreciation
**Priority:** T2
**Complexity:** M
**Dependencies:** BLDG-006
**Source:** urban_planning_zoning.md, section 3.5; economic_simulation.md, section 3.1

## Description
Track building age and apply depreciation to building value over time. Old buildings have lower value, quality, and eventually filter to lower-income occupants. This is the core mechanism behind neighborhood lifecycle and housing filtering.

- Add `construction_day: u32` to Building component
- Building value = base_value * (1 - depreciation_rate * age_years).max(0.1)
- Depreciation rates: Residential 1.5%, Commercial 2.5%, Industrial 4%, Office 2%
- Building quality (BLDG-004) degrades with age (old buildings need more maintenance)
- Visual weathering: BuildingAppearance.weathering increases with age
- Buildings older than 50 years become "filtering" candidates for lower-income residents

## Definition of Done
- [ ] Building tracks construction_day
- [ ] Depreciation formula reduces building value over time
- [ ] Old buildings have lower quality scores
- [ ] Visual weathering reflects age
- [ ] Save/load preserves construction_day

## Test Plan
- Unit: Building aged 0 has value == base_value
- Unit: Building aged 50 years has value < 50% of base
- Integration: Buildings age over game time, verify visual weathering changes

## Pitfalls
- LifeSimTimer serialization bug (already known) affects building age tracking
- Need to set construction_day for existing buildings on save migration
- Depreciation should not apply during construction (only after completed)

## Relevant Code
- `crates/simulation/src/buildings.rs:Building` -- add construction_day
- `crates/simulation/src/building_upgrade.rs` -- age affects quality score
- `crates/rendering/src/building_render.rs` -- weathering visual
- `crates/save/src/serialization.rs` -- serialize construction_day
