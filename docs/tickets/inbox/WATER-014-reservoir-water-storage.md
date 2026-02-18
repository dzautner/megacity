# WATER-014: Reservoir Water Level and Storage Mechanics

## Priority: T2 (Depth)

## Description
Implement reservoir water level tracking with inflow (rain, river), outflow (supply), evaporation, and storage capacity. Reservoir level affects available water supply and provides a buffer during droughts.

## Current State
- No reservoir building or resource.
- Water supply is instantaneous (no storage concept).

## Definition of Done
- [ ] `Reservoir` component with: `capacity_mg`, `current_level_mg`, `inflow_rate`, `outflow_rate`.
- [ ] Inflow: `rainfall * catchment_area + river_flow`.
- [ ] Outflow: water supply extraction (capped by treatment capacity).
- [ ] Evaporation: `surface_area * evaporation_rate * temperature_factor`.
- [ ] Storage days = current_level / daily_demand.
- [ ] Minimum level: 20% reserve for emergency.
- [ ] Warning events at 50%, 30%, 20% levels.
- [ ] Visual: reservoir fills/empties in terrain rendering.

## Test Plan
- [ ] Unit test: reservoir fills during rainy season.
- [ ] Unit test: reservoir depletes during drought.
- [ ] Unit test: evaporation increases in summer.
- [ ] Integration test: reservoir provides 90-day buffer.

## Pitfalls
- Catchment area and inflow calculation require terrain analysis.
- Evaporation losses are significant in hot climates.
- Reservoir visual in terrain requires new rendering.

## Code References
- Research: `environment_climate.md` section 2.2
