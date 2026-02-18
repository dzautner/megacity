# INFRA-055: Five-Level Building Upgrade System
**Priority:** T1
**Complexity:** M (1-3 days)
**Dependencies:** none
**Source:** master_architecture.md, M2

## Description
Expand building levels from 3 to 5 with clear visual progression. Level thresholds based on land value and service coverage. L1: basic structure, L2: improved (land value >100), L3: developed (>150, requires water+power), L4: high-density (>200, requires transit access), L5: landmark quality (>250, requires full services). Each level increases population/employment capacity, tax revenue, and visual height. Building upgrade criteria in `building_upgrade.rs`.

## Definition of Done
- [ ] Buildings support levels 1-5
- [ ] Land value thresholds for each level
- [ ] Service requirements for L3+ (water, power, transit)
- [ ] Population/employment capacity scales with level
- [ ] Visual mesh changes per level (at least 2 variants per level per zone type)
- [ ] Tests pass

## Test Plan
- Unit: Building at land_value=250 with all services upgrades to L5
- Unit: Building without water service cannot reach L3
- Integration: High-value downtown areas show tall L5 buildings

## Pitfalls
- Current thresholds use raw u8 land value; need to rescale
- Upgrade path: L3->L4 requires transit which may not exist early game
- Downgrade logic: if services removed, building should eventually downgrade

## Relevant Code
- `crates/simulation/src/building_upgrade.rs` -- upgrade criteria
- `crates/simulation/src/buildings.rs` -- building levels, capacity
- `crates/rendering/src/building_meshes.rs` -- mesh per level
