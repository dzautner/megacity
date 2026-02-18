# POWER-001: Per-Building Energy Demand Calculation

## Priority: T1 (Core)

## Description
Implement per-building electricity demand calculation with time-of-day variation. The research doc specifies base demand by building type (residential, commercial, industrial) with seasonal multipliers (heating/cooling) and HDD/CDD-driven HVAC load scaling.

## Current State
- `Weather::power_multiplier()` provides seasonal scaling (Winter 1.4, Summer 1.2).
- No per-building demand calculation.
- No time-of-day variation.
- No total demand vs supply tracking.
- No blackout system.

## Definition of Done
- [ ] `EnergyConsumer` component with `base_demand_kwh: f32`, `priority: LoadPriority`.
- [ ] Demand rates: residential=1,000 kWh/month base, commercial low=3,000, commercial high=15,000, industrial=50,000, hospital=200,000, data center=500,000.
- [ ] HDD/CDD modifier: `daily_hvac_modifier = 1.0 + HDD*0.02 + CDD*0.03`.
- [ ] Time-of-use demand curve: off-peak(22-06)=0.6, mid(06-14)=1.0, on-peak(14-22)=1.5.
- [ ] `EnergyGrid` resource with `total_demand_mwh`, `total_supply_mwh`, `reserve_margin`.
- [ ] Demand aggregation system running every 4 ticks.

## Test Plan
- [ ] Unit test: residential demand = 1000 kWh/month base.
- [ ] Unit test: winter demand is 40% higher than base (from power_multiplier).
- [ ] Unit test: on-peak demand is 50% higher than mid-peak.
- [ ] Integration test: city of 10,000 citizens demands ~3-5 MW base load.

## Pitfalls
- Time-of-day demand requires `GameClock.hour` which must be available.
- HDD/CDD calculation requires average daily temperature from weather system.
- Must define LoadPriority levels for rolling blackouts (Critical/Essential/Standard/Deferrable).

## Code References
- `crates/simulation/src/weather.rs`: `Weather::power_multiplier()`
- `crates/simulation/src/time_of_day.rs`: `GameClock`
- Research: `environment_climate.md` sections 3.1, 3.5.1
