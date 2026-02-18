# INFRA-049: Parking Requirements per Building
**Priority:** T3
**Complexity:** M (1-3 days)
**Dependencies:** none
**Source:** transportation_simulation.md, Section 6.1

## Description
Add parking space tracking per building. Parking requirement by zone type: ResidentialLow 2.0 spaces/unit, ResidentialHigh 1.0, CommercialLow 3.0/100sqm, CommercialHigh 2.0, Industrial 1.0, Office 2.5. Transit-oriented development (within 400m of transit stop) gets 50% reduction. Higher density levels get 40-100% of base requirement. Auto-calculate parking type by level: surface (L1-2), structure (L3), underground (L4-5).

## Definition of Done
- [ ] `parking_requirement()` function per zone type, level, transit proximity
- [ ] Parking type auto-selected by zone level
- [ ] Parking spaces stored per building
- [ ] Parking utilization tracked (demand vs supply)
- [ ] Tests pass

## Test Plan
- Unit: ResidentialLow L1 has 2.0 spaces/unit; near transit = 1.0
- Unit: CommercialHigh L5 near transit = 0.4 spaces/100sqm

## Pitfalls
- Parking space calculation must be retroactive when transit stop placed nearby
- Buildings built before transit should not auto-reduce parking

## Relevant Code
- `crates/simulation/src/buildings.rs` -- building properties
- `crates/simulation/src/zones.rs` -- zone type
