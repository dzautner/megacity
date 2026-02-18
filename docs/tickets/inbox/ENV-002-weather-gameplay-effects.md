# ENV-002: Weather Gameplay Effects
**Priority:** T2
**Complexity:** M
**Dependencies:** none
**Source:** master_architecture.md, section M3; cities_skylines_analysis.md, section 14.2

## Description
Weather should have gameplay consequences beyond visuals. Storms damage buildings, snow slows traffic, rain increases stormwater runoff, heat waves increase water demand and health risks.

- Storm: lightning can start fires, heavy rain reduces visibility/speed
- Snow: roads slowed by 40% without plowing, accidents increase, heating demand increases
- Heat wave: water demand +30%, health risk for elderly, power demand for AC increases
- Rain: stormwater runoff increases flood risk (if stormwater system implemented)
- Cold: heating demand increases, pipes can burst if temperature too low
- Fog: no gameplay effect but visual atmosphere

## Definition of Done
- [ ] Storm damage (lightning fires)
- [ ] Snow speed reduction on unplowed roads
- [ ] Heat wave water/power demand increase
- [ ] Weather effects visible in gameplay, not just visuals

## Test Plan
- Integration: Storm occurs, verify fire starts from lightning
- Integration: Snow covers roads, verify traffic speed reduced

## Pitfalls
- weather.rs already has seasonal cycles but no gameplay effects
- Must not make weather too punishing (casual players frustrated by constant crises)
- Snow plowing (road_maintenance.rs) needs vehicle dispatch

## Relevant Code
- `crates/simulation/src/weather.rs` -- weather state
- `crates/simulation/src/fire.rs` -- lightning fire starts
- `crates/simulation/src/traffic.rs` -- weather speed modifier
- `crates/simulation/src/heating.rs` -- cold weather heating demand
