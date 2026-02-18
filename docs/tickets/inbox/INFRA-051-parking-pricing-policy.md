# INFRA-051: Parking Pricing Policies
**Priority:** T5
**Complexity:** M (1-3 days)
**Dependencies:** INFRA-049, INFRA-050
**Source:** transportation_simulation.md, Section 6.3

## Description
Implement parking pricing as a policy tool. Policy options: FreeParking (default), MeteredStreet ($1-5/hr), ResidentialPermit ($50-200/yr), DemandPricing (target 85% occupancy, variable rate), ParkingMaximum (cap spaces/unit). Pricing affects auto mode utility in mode choice model. Revenue from metered parking. Demand elasticity: 10% price increase -> 1-3% fewer parkers (commuter), 3-6% (shopping).

## Definition of Done
- [ ] `ParkingPolicy` enum with 5 variants
- [ ] Parking cost fed into mode choice auto utility
- [ ] Revenue calculation from metered parking
- [ ] Demand elasticity reduces parking demand
- [ ] Per-district parking policy setting
- [ ] Tests pass

## Test Plan
- Unit: Free to $5/day: 10-15% traffic reduction
- Unit: Metered parking generates revenue
- Integration: Demand pricing stabilizes occupancy at 60-80%

## Pitfalls
- Revenue calculation requires occupancy tracking
- Demand pricing needs real-time adjustment (feedback loop)
- Player confusion if parking policy effects are invisible

## Relevant Code
- `crates/simulation/src/policies.rs` -- policy framework
- `crates/simulation/src/economy.rs` -- parking revenue
- `crates/simulation/src/districts.rs` -- per-district policies
