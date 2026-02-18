# WATER-009: Flood Levee and Seawall Infrastructure

## Priority: T3 (Differentiation)

## Description
Implement flood protection infrastructure: levees, seawalls, and floodgates. Levees prevent river flooding up to their design capacity. Seawalls protect against coastal surge. Both have maintenance requirements and can fail if overtopped.

## Current State
- No flood protection infrastructure.
- Floods are a direct terrain-based disaster.

## Definition of Done
- [ ] Levee building: placeable along rivers, prevents flooding up to design height (10 ft default).
- [ ] Seawall: placeable along coast, prevents coastal surge up to design height.
- [ ] Floodgate: allows controlled water release, prevents upstream flooding.
- [ ] Overtopping: if flood exceeds design height, levee fails and flooding is worse than without it.
- [ ] Maintenance: levees require $2K/cell/year maintenance; neglected levees degrade.
- [ ] Levee failure probability: increases with age and lack of maintenance.
- [ ] Flood zone recalculation after levee placement (DISASTER-006).

## Test Plan
- [ ] Unit test: levee blocks flood below design height.
- [ ] Unit test: flood above design height causes levee failure.
- [ ] Integration test: levee-protected area stays dry during moderate flood.
- [ ] Integration test: levee failure causes catastrophic flooding.

## Pitfalls
- Levee failure makes flooding worse (overconfidence effect).
- Flood zone recalculation after levee placement is expensive.
- Players may rely entirely on levees and be devastated by overtopping.

## Code References
- Research: `environment_climate.md` sections 5.2.2, 5.2.3
