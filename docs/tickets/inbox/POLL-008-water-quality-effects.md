# POLL-008: Water Quality Effects on Citizens and Fisheries

## Priority: T2 (Depth)

## Description
Implement the 6-tier water quality effects system from the research doc. Currently water pollution health penalty uses a simple threshold (>50 = penalty). The research doc defines tiers from Pristine (health bonus, tourism bonus) to Toxic (health -0.20, water unusable), plus fishery yield effects and drinking water treatment cost scaling.

## Current State
- `water_pollution_health_penalty` applies a linear penalty for pollution > 50.
- No health bonus for clean water.
- No fishery yield system.
- No drinking water treatment cost scaling.
- No swimming/recreation effects.

## Definition of Done
- [ ] 6-tier water quality classification: Pristine/Clean/Moderate/Polluted/Heavy/Toxic.
- [ ] Health effects per tier matching research doc values.
- [ ] Health bonus (+0.02) for citizens near pristine water.
- [ ] Tourism bonus for pristine water areas.
- [ ] Drinking water treatment cost: scales from $500/MG (clean) to $5000/MG (very polluted).
- [ ] Visual discoloration of water cells based on pollution level in terrain rendering.

## Test Plan
- [ ] Unit test: each tier boundary returns correct health modifier.
- [ ] Unit test: pristine water provides positive health effect.
- [ ] Integration test: building near polluted water reduces citizen health.
- [ ] Integration test: treating polluted water source costs more than clean source.

## Pitfalls
- Fishery system does not exist yet; can stub the yield modifier for later use.
- Water rendering discoloration requires changes to `terrain_render.rs`.
- Treatment cost scaling needs to integrate with the economy/budget system.

## Code References
- `crates/simulation/src/water_pollution.rs`: `water_pollution_health_penalty`
- `crates/rendering/src/terrain_render.rs`: water cell rendering
- Research: `environment_climate.md` section 1.2.5
