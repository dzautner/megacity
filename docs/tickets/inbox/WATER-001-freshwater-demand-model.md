# WATER-001: Freshwater Demand Calculation Per Building Type

## Priority: T1 (Core)

## Description
Implement per-building-type water demand calculation. The research doc specifies base demand of 150 gallons per capita per day (GPCD), varying by building category: residential, commercial, industrial, hospitals, schools, parks, fire hydrants, cooling towers, and construction sites.

## Current State
- `Weather::water_multiplier()` provides seasonal scaling but no actual demand calculation.
- No per-building water demand system.
- No water supply/demand balance tracking.
- `GroundwaterGrid` exists but is not connected to demand.

## Definition of Done
- [ ] `WaterDemand` component on each building with `demand_gpd: f32`.
- [ ] Per-type demand rates: residential=150 GPCD, commercial=100 GPB, industrial=500 GPB, hospital=300, school=25 per student, park/irrigation=500 per cell.
- [ ] Seasonal modifiers from `Weather::water_multiplier()` applied.
- [ ] `WaterSupply` resource tracking total demand vs total supply.
- [ ] Demand aggregation system computing city-wide totals each game-day.
- [ ] Service deficit: buildings without water service have reduced happiness.

## Test Plan
- [ ] Unit test: residential building demand = occupants * 150 GPCD.
- [ ] Unit test: summer demand is 30% higher than base.
- [ ] Integration test: city with 10,000 citizens demands ~1.5 MGD base.
- [ ] Integration test: no water infrastructure = service deficit warning.

## Pitfalls
- Gallons per capita per day (GPCD) is a US-centric unit; UI should also show metric.
- Must define what "no water service" means in terms of building placement rules.
- Water demand affects groundwater drawdown (interaction with `groundwater.rs`).

## Code References
- `crates/simulation/src/weather.rs`: `Water::water_multiplier()`
- `crates/simulation/src/groundwater.rs`: `GroundwaterGrid`
- Research: `environment_climate.md` section 2.1
