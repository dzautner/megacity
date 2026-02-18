# WEATHER-003: Heating/Cooling Degree Days (HDD/CDD) Energy Impact

## Priority: T2 (Depth)

## Description
Implement Heating Degree Days and Cooling Degree Days as the driver for HVAC energy demand. HDD = max(0, 65F - T_avg), CDD = max(0, T_avg - 65F). Each HDD adds 2% heating load, each CDD adds 3% cooling load. This connects temperature directly to energy demand in a realistic way.

## Current State
- `Weather::power_multiplier()` provides a simple seasonal multiplier (Winter 1.4, Summer 1.2).
- `heating.rs` has `heating_demand()` function with linear ramp from 10C to -10C.
- No CDD-based cooling calculation.
- No HDD tracking over time.

## Definition of Done
- [ ] `HDD = max(0, 65F - T_avg_F)` computed daily from weather temperature.
- [ ] `CDD = max(0, T_avg_F - 65F)` computed daily.
- [ ] `daily_hvac_modifier = 1.0 + HDD * 0.02 + CDD * 0.03`.
- [ ] HVAC modifier applied to residential and commercial energy demand.
- [ ] Monthly HDD/CDD tracking for UI statistics display.
- [ ] Cumulative annual HDD/CDD for climate characterization.
- [ ] High energy costs from extreme HDD/CDD reduce citizen happiness (-0.05 per $100/month above baseline).

## Test Plan
- [ ] Unit test: 30 HDD day = 60% more energy demand.
- [ ] Unit test: 20 CDD day = 60% more energy demand.
- [ ] Unit test: 0 HDD and 0 CDD = 1.0x modifier (no extra demand).
- [ ] Integration test: cold snap with 50 HDD doubles energy demand.
- [ ] Integration test: heat wave with 30 CDD significantly increases demand.

## Pitfalls
- Weather temperature is in Celsius; HDD/CDD baseline is 65F (18.3C). Need conversion.
- HDD/CDD modifier stacks with existing `power_multiplier()`; may need to replace it.
- Citizens without heating/cooling (no heating plant coverage) should have worse health effects.

## Code References
- `crates/simulation/src/weather.rs`: `Weather.temperature`
- `crates/simulation/src/heating.rs`: `heating_demand()`
- Research: `environment_climate.md` sections 4.2.1-4.2.3
