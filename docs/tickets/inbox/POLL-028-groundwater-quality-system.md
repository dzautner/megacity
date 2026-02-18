# POLL-028: Groundwater Quality System Enhancement

## Priority: T2 (Depth)

## Description
Enhance the existing groundwater quality tracking with contamination sources, plume spread, and remediation. Currently quality degrades near industrial areas and pollution. The research doc specifies soil contamination seeping into groundwater, landfill leachate, and treatment plant effectiveness on groundwater.

## Current State
- `GroundwaterGrid` has `quality` field (0-255) per cell.
- Quality degrades near industrial zones and pollution sources.
- `WaterTreatmentPlant` purifies groundwater in its radius.
- Quality affects citizen health (penalty when quality < 50).

## Definition of Done
- [ ] Contamination sources: soil contamination (POLL-013) seeps into groundwater at 0.1x rate.
- [ ] Landfill leachate: unlined landfill degrades quality in radius 10, lined = radius 3.
- [ ] Industrial discharge: quality degrades within radius 5 of industrial buildings.
- [ ] Groundwater flow: contamination spreads in direction of groundwater flow.
- [ ] Treatment plant effectiveness: quality recovery rate based on treatment level.
- [ ] Drinking water quality: average groundwater quality at well locations determines water quality tier.

## Test Plan
- [ ] Unit test: soil contamination seeps into groundwater at correct rate.
- [ ] Unit test: unlined landfill degrades groundwater in larger radius than lined.
- [ ] Integration test: well near landfill produces lower-quality water.
- [ ] Integration test: treatment plant improves groundwater quality over time.

## Pitfalls
- Groundwater flow direction requires aquifer modeling (simplified to terrain slope).
- Quality improvements from treatment must be slower than degradation to feel meaningful.
- Must integrate with WATER-003 (water treatment levels).

## Code References
- `crates/simulation/src/groundwater.rs`: `GroundwaterGrid`, quality tracking
- `crates/simulation/src/water_pollution.rs`: surface water interaction
- Research: `environment_climate.md` sections 1.2, 1.4
