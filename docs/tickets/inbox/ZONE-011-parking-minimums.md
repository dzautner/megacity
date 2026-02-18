# ZONE-011: Parking Minimum/Maximum System
**Priority:** T3
**Complexity:** M
**Dependencies:** ZONE-005
**Source:** urban_planning_zoning.md, section 6.6

## Description
Implement parking requirements as a zoning control. Parking minimums consume land and increase construction costs. Parking maximums encourage transit use. Modeled after Donald Shoup's research.

- Per-zone parking ratios: Residential 1-2 per unit, Commercial 1 per 300 sqft, Industrial 1 per 500 sqft
- Parking requirement increases effective building cost (+$5,000-$20,000 per required space)
- Surface parking lots consume land cells (big-box commercial needs 8-16 cells parking)
- Policy toggle: eliminate parking minimums (reduces cost, increases transit dependency)
- Parking maximum policy: cap parking to encourage transit
- Underground parking implied for high-density (no land consumption but higher cost)

## Definition of Done
- [ ] Parking requirements calculated per building based on zone and capacity
- [ ] Parking cost added to building construction cost
- [ ] Policy to eliminate parking minimums exists
- [ ] Visual: surface parking lots for commercial zones

## Test Plan
- Unit: CommercialLow building parking requirement > 0
- Unit: Eliminating parking minimums reduces construction cost
- Integration: Build commercial with parking minimum, verify higher construction delay/cost

## Pitfalls
- Surface parking lots need multi-cell building support
- Interaction with parking simulation (T5 feature) -- this is the zoning layer, not the vehicle layer
- Don't make parking minimums so expensive that no commercial builds

## Relevant Code
- `crates/simulation/src/buildings.rs` -- add parking cost to construction
- `crates/simulation/src/policies.rs` -- parking minimum policy toggle
- `crates/simulation/src/grid.rs:ZoneType` -- parking ratio per zone
