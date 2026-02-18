# DISASTER-002: Building Construction Types for Earthquake Vulnerability

## Priority: T2 (Depth)

## Description
Add construction type to buildings that determines earthquake vulnerability. The research doc defines 5 construction types (Wood Frame, Unreinforced Masonry, Reinforced Concrete, Steel Frame, Seismic-Designed) with different damage probabilities at each MMI level.

## Current State
- Buildings have no construction type attribute.
- Earthquake damage is a flat 10% destruction chance regardless of building type.
- `Building` has `zone_type` and `level` but no `construction_type`.

## Definition of Done
- [ ] `ConstructionType` enum: WoodFrame, UnreinforcedMasonry, ReinforcedConcrete, SteelFrame, SeismicDesigned.
- [ ] Default construction type based on zone type and level: low-density=WoodFrame, industrial=Masonry, high-rise=ReinforcedConcrete or Steel.
- [ ] Damage probability table: 4 states (None/Moderate/Severe/Collapse) per MMI level per construction type.
- [ ] `DamageState` enum applied per building after earthquake.
- [ ] Repair cost: Moderate=10-30% building value, Severe=50-80%, Collapse=100%.
- [ ] Casualty rate: Collapse=5-15% of occupants, Severe=1%.
- [ ] Seismic building code policy: makes all new buildings SeismicDesigned (+20% build cost).
- [ ] Retrofit program: upgrade existing buildings to seismic standard ($10K/building).

## Test Plan
- [ ] Unit test: WoodFrame at MMI VIII = 45% None, 35% Moderate, 15% Severe, 5% Collapse.
- [ ] Unit test: SeismicDesigned at MMI VIII = 90% None, 8% Moderate, 2% Severe, 0% Collapse.
- [ ] Integration test: M7 earthquake destroys masonry buildings but spares seismic-designed ones.
- [ ] Integration test: seismic building code reduces total damage from subsequent earthquakes.

## Pitfalls
- Adding construction type to all buildings requires save/load migration.
- Damage probability table has 40+ entries; must be data-driven, not hardcoded.
- Retrofit program needs to iterate through all existing buildings (potentially expensive).

## Code References
- `crates/simulation/src/buildings.rs`: `Building` component
- `crates/simulation/src/disasters.rs`: earthquake damage
- Research: `environment_climate.md` sections 5.1.3-5.1.5
