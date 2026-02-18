# INFRA-145: Parking Structures and Land Use
**Priority:** T5
**Complexity:** M (1-3 days)
**Dependencies:** INFRA-049
**Source:** transportation_simulation.md, Section 6.4

## Description
Implement parking buildings: surface lots (30-35 sqm/space, $5-15K/space), above-ground structures (25-30 sqm/space, $25-50K/space), underground garages (30-35 sqm/space, $35-75K/space). Auto-select parking type by zone density level: L1-2 surface, L3 structure, L4-5 underground. Surface parking consumes land (100-space lot = 3000+ sqm, footprint of a 3-story apartment). Parking-density paradox: medium density areas trapped by surface parking preventing densification.

## Definition of Done
- [ ] Parking facility building types (surface, structure, underground)
- [ ] Cost and space per type
- [ ] Auto-selection by zone density level
- [ ] Surface lots consume buildable land
- [ ] Parking capacity per building displayed
- [ ] Tests pass

## Test Plan
- Unit: Surface lot occupies 1+ grid cells
- Unit: Underground garage does not consume surface land
- Integration: Dense downtown uses underground parking, suburbs use surface

## Pitfalls
- Surface parking lots as separate buildings or auto-generated with zones?
- Underground parking conflicts with underground infrastructure (INFRA-023)
- Parking structures have NIMBY effect (traffic, visual)

## Relevant Code
- `crates/simulation/src/buildings.rs` -- parking building types
