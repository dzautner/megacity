# INFRA-139: Historic Preservation Districts
**Priority:** T3
**Complexity:** S (hours)
**Dependencies:** INFRA-138
**Source:** master_architecture.md, M4

## Description
Allow player to designate historic preservation districts. Buildings in these districts cannot be demolished or upgraded (preserving their era appearance). Historic districts provide tourism and land value bonus. Restrictions: no new construction styles that don't match the district era. Trade-off: preservation limits density growth but increases cultural value.

## Definition of Done
- [ ] Historic district designation tool
- [ ] Demolition and upgrade blocked in historic districts
- [ ] Tourism and land value bonus for historic districts
- [ ] Building style enforcement matching district era
- [ ] Tests pass

## Test Plan
- Unit: Building in historic district cannot be demolished
- Unit: Land value increases after historic designation
- Integration: Historic downtown attracts tourists

## Pitfalls
- Player may accidentally lock themselves out of upgrading critical areas
- Historic designation should be reversible (with citizen opposition)

## Relevant Code
- `crates/simulation/src/districts.rs` -- district types
- `crates/simulation/src/tourism.rs` -- tourism bonus
