# WATER-007: Wastewater and Sewage Collection System

## Priority: T2 (Depth)

## Description
Implement a wastewater collection system where buildings generate sewage at 80% of their water consumption. Sewage must be routed to treatment plants; uncollected sewage causes health and pollution problems.

## Current State
- `UtilityType::SewagePlant` exists but has no sewage flow model.
- No sewage generation per building.
- No sewage coverage/capacity tracking.
- No penalties for lack of sewage service.

## Definition of Done
- [ ] `WastewaterProduction` component: `sewage_gpd = water_demand * 0.80`.
- [ ] Sewage service coverage: buildings within range of sewage plant are serviced.
- [ ] `SewageSystem` resource tracking total generation vs treatment capacity.
- [ ] Untreated sewage: if no capacity, raw sewage discharges to nearest water body (pollution source).
- [ ] Health penalty for uncollected sewage near residential areas.
- [ ] Combined vs separated sewer option (policy): combined overflows during rain.

## Test Plan
- [ ] Unit test: sewage = 80% of water demand for each building.
- [ ] Unit test: sewage plant covers buildings within its service radius.
- [ ] Integration test: city without sewage treatment has high water pollution.
- [ ] Integration test: combined sewer overflows during heavy rain.

## Pitfalls
- Separated vs combined sewer is a major infrastructure difference; may want to simplify for first pass.
- Raw sewage discharge locations need to be auto-determined (nearest water cell).
- Sewage treatment capacity must match city growth or create a crisis.

## Code References
- `crates/simulation/src/utilities.rs`: `UtilityType::SewagePlant`
- `crates/simulation/src/water_pollution.rs`: discharge integration
- Research: `environment_climate.md` sections 2.5, 2.6
