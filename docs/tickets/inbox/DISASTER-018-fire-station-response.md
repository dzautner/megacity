# DISASTER-018: Fire Station Response to Building and Wildfire

## Priority: T2 (Depth)

## Description
Enhance fire station response to both building fires and wildfires. Fire stations should deploy units to nearest fire, with response time affecting damage. Firefighter units actively suppress fire intensity and contain spread. Water supply (hydrants) required for effective firefighting.

## Current State
- `extinguish_fires` in `fire.rs` reduces fire grid values in service radius.
- No unit deployment or pathfinding.
- No water supply requirement.
- No response time calculation.

## Definition of Done
- [ ] Fire unit deployment: each station has N units, each assigned to nearest fire.
- [ ] Response time: `time = distance_to_fire / unit_speed`.
- [ ] Active suppression: `fire_intensity *= 0.5` per firefighter at cell.
- [ ] Containment: spread probability from suppressed cell reduced by 70%.
- [ ] Water supply: hydrant within 5 cells required; without, firefighting at 30% effectiveness.
- [ ] Water consumption: 500 gal/min per fire cell being fought.
- [ ] Multiple fires: units split between fires, reducing per-fire effectiveness.
- [ ] Wildfire integration: fire stations respond to forest fires within service radius.

## Test Plan
- [ ] Unit test: closer fire station responds faster.
- [ ] Unit test: no hydrant reduces effectiveness to 30%.
- [ ] Integration test: fire station contains small building fire.
- [ ] Integration test: wildfire near city triggers fire station response.

## Pitfalls
- Unit pathfinding to fire is expensive; simplify to direct distance.
- Water consumption during firefighting draws from water supply (WATER system interaction).
- Multiple simultaneous fires can overwhelm fire service capacity.

## Code References
- `crates/simulation/src/fire.rs`: `extinguish_fires`
- `crates/simulation/src/forest_fire.rs`: wildfire system
- Research: `environment_climate.md` section 5.3.6 (Firefighting)
