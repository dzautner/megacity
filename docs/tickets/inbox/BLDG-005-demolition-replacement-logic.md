# BLDG-005: Market-Driven Demolition and Replacement
**Priority:** T2
**Complexity:** L
**Dependencies:** BLDG-001, ECON-006
**Source:** urban_planning_zoning.md, section 3.5

## Description
Implement economic redevelopment logic. When land value exceeds building value + demolition cost, the building should be demolished and replaced with a denser/higher-value building. This creates natural urban renewal and density transitions.

- Building value = construction_cost * (1 - depreciation_rate * age)
- Depreciation: Residential 1.5%/year, Commercial 2.5%/year, Industrial 4%/year, Office 2%/year
- Demolition cost = 15% of original construction cost
- Redevelopment triggers when: land_value > building_value + demolition_cost AND demand > 0.3 AND max_viable_level > current_level
- Run evaluation on 1% of buildings per slow tick (random subset)
- Demolition creates "rubble" state (30 game days) before cell available again
- Track building age (in game-days since construction)

## Definition of Done
- [ ] Building component tracks construction date / age
- [ ] Depreciation reduces building value over time
- [ ] Redevelopment evaluation runs periodically on random subset
- [ ] Buildings demolished when economically viable
- [ ] Rubble period before reconstruction
- [ ] New building spawns at appropriate level for current conditions

## Test Plan
- Unit: Building aged 50 years has value < 50% of original
- Unit: Redevelopment triggers when land_value >> building_value
- Integration: Increase land value in area with old buildings, verify gradual replacement

## Pitfalls
- Must evict occupants before demolition (citizens need new homes)
- Rubble state needs visual feedback (construction debris mesh)
- Rapid redevelopment in hot areas can cause citizen displacement cascades
- Building age must be serialized

## Relevant Code
- `crates/simulation/src/buildings.rs:Building` -- add construction_day field
- `crates/simulation/src/building_upgrade.rs` -- add redevelopment evaluation
- `crates/simulation/src/abandonment.rs` -- rubble state handling
- `crates/simulation/src/land_value.rs` -- land value comparison
