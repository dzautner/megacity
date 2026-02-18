# CIT-059: Five-Level Building System

**Priority:** T2 (Depth)
**Complexity:** Medium (2-3 person-weeks)
**Dependencies:** None
**Source:** master_architecture.md Section 1.4

## Description

Expand building levels from 3 to 5 (matching CS1). Level thresholds based on land value + service coverage + education + pollution. Level 1: base (land value > 0). Level 2: land value > 80, has power+water. Level 3: land value > 120, has health+education. Level 4: land value > 160, has all services, low pollution. Level 5: land value > 200, max services, educated workforce, low crime. Each level has increased capacity, different mesh, higher property tax value.

## Definition of Done

- [ ] Building.level range extended to 1-5
- [ ] Level-up thresholds for levels 4 and 5
- [ ] Service coverage requirements per level
- [ ] Pollution maximum per level (decreasing)
- [ ] Education quality requirement for level 5
- [ ] Capacity multipliers per level
- [ ] Mesh variants for levels 4 and 5
- [ ] Property tax value increases with level

## Test Plan

- Unit test: level 4 requires land value > 160 + all services
- Unit test: level 5 requires educated workforce
- Unit test: pollution prevents level-up
- Integration test: well-served area produces level 5 buildings

## Pitfalls

- Current system uses u8 for level; extending to 5 is compatible
- Level 5 should be aspirational (not common)

## Relevant Code

- `crates/simulation/src/buildings.rs` (Building.level)
- `crates/simulation/src/building_upgrade.rs` (upgrade_buildings, thresholds)
- `crates/rendering/src/building_meshes.rs` (mesh generation)
