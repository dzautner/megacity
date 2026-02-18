# DISASTER-008: Firebreak Types and Effectiveness

## Priority: T3 (Differentiation)

## Description
Implement firebreak effectiveness for different terrain types. Paved roads, highways, rivers, cleared ground, irrigated green spaces, and concrete buildings all act as firebreaks with different stop probabilities.

## Current State
- `forest_fire.rs` reduces spread across roads (spread_chance /= 4) but no formal firebreak system.
- Water cells block fire completely.
- No maintained firebreak building type.
- No firebreak overlay.

## Definition of Done
- [ ] Paved road (2 lanes): 70% stop probability, 1 cell width.
- [ ] Highway (4+ lanes): 90% stop, 2 cells width.
- [ ] River/water: 95% stop, 1 cell.
- [ ] Cleared/bare ground: 85% stop, 2 cells width.
- [ ] Irrigated green: 75% stop, 1 cell.
- [ ] Concrete buildings: 60% stop for grass/brush fires.
- [ ] Maintained firebreak: placeable building type, 85% stop, requires annual maintenance $1K/cell.
- [ ] Firebreak overlay showing fire resistance per cell.

## Test Plan
- [ ] Unit test: highway stops 90% of fire spread attempts.
- [ ] Unit test: water stops 95% of fire spread.
- [ ] Integration test: fire approaching a highway is mostly contained.
- [ ] Integration test: maintained firebreak ring around city prevents wildfire entry.

## Pitfalls
- Ember jumping can bypass firebreaks (realistic but may frustrate players).
- Maintained firebreaks need annual maintenance cost tracking.
- Stop probability must integrate with the existing spread hash system.

## Code References
- `crates/simulation/src/forest_fire.rs`: spread through roads
- Research: `environment_climate.md` section 5.3.7
