# WASTE-003: Waste Collection System and Service Areas

## Priority: T1 (Core)

## Description
Implement waste collection as a service-area system. Collection facilities (transfer stations) have a service radius and truck-based capacity. When waste exceeds collection capacity, uncollected waste accumulates, causing health and happiness penalties.

## Current State
- No waste collection infrastructure.
- No service area for waste.
- No uncollected waste tracking.

## Definition of Done
- [ ] Transfer station: 200 tons/day capacity, 2x2 footprint, $500K build, $2K/day operating.
- [ ] Service radius: 20 cells from transfer station.
- [ ] Collection capacity: trucks * 10 tons/truck * 3-4 trips/day.
- [ ] Collection rate: `min(1.0, capacity / waste_generated)`.
- [ ] Uncollected waste: accumulates at buildings, health penalty, happiness -5, land value -10%.
- [ ] Transport cost: `total_waste * cost_per_ton_mile * avg_distance`.
- [ ] Collection route: closer buildings served first (implicit, not simulated per-truck).
- [ ] `WasteSystem` tracks `total_collected_tons` vs `total_generated_tons`.

## Test Plan
- [ ] Unit test: transfer station serves buildings within 20 cells.
- [ ] Unit test: collection at 80% capacity means 20% uncollected.
- [ ] Integration test: placing a transfer station reduces uncollected waste.
- [ ] Integration test: uncollected waste causes happiness penalty.

## Pitfalls
- Service area overlap between multiple transfer stations needs handling (don't double-count).
- Truck routes are simplified to capacity-based; no per-truck pathfinding.
- Distance-based cost favors distributed facilities (good incentive design).

## Code References
- `crates/simulation/src/services.rs`: service coverage pattern
- Research: `environment_climate.md` sections 6.2.1-6.2.2
