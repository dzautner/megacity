# DISASTER-009: Wildfire Mitigation and Firefighting

## Priority: T3 (Differentiation)

## Description
Implement wildfire mitigation measures: prescribed burns, fire-resistant building codes, defensible space requirements, water infrastructure (hydrants), aerial firefighting, and evacuation routes. Also implement firefighter deployment that actively suppresses fires.

## Current State
- Fire stations provide coverage that reduces fire intensity (`extinguish_fires` in `fire.rs`).
- No prescribed burn system.
- No defensible space concept.
- No aerial firefighting.
- No fire weather monitoring.

## Definition of Done
- [ ] Prescribed burns: player-initiated, reduces fuel load by 80% in target area, $500/cell.
- [ ] Fire-resistant building code: reduces structure damage 60%, +15% build cost.
- [ ] Defensible space: 100ft (6 cells) cleared around structures, $2K/building.
- [ ] Hydrant coverage: required for firefighter water supply, $5K/cell.
- [ ] Aerial firefighting: deployable during active fire, $50K per deployment, drops water on cells.
- [ ] Evacuation routes: road planning reduces casualties 80%.
- [ ] Fire weather monitoring: $20K, provides red-flag day warnings.
- [ ] Active firefighting: fire station units deploy to nearest burning cell, halve spread rate.

## Test Plan
- [ ] Unit test: prescribed burn sets fuel level to 20% of original.
- [ ] Unit test: aerial firefighting reduces fire intensity by 50 per drop.
- [ ] Integration test: fire station actively suppresses nearby fire.
- [ ] Integration test: defensible space prevents structure ignition.

## Pitfalls
- Prescribed burns could accidentally spread if wind shifts.
- Aerial firefighting requires water supply (WATER-001 dependency).
- Many overlapping mitigation measures; need clear UI organization.

## Code References
- `crates/simulation/src/fire.rs`: fire extinguishing
- `crates/simulation/src/forest_fire.rs`: forest fire system
- Research: `environment_climate.md` section 5.3.8
