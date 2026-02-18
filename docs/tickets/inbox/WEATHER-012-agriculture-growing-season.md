# WEATHER-012: Agricultural Growing Season and Crop Yield

## Priority: T3 (Differentiation)

## Description
Implement a growing season system for agricultural zones. Crops only grow when temperature > 50F, frost risk < 10%, and not winter. Crop yield depends on rainfall adequacy, temperature suitability, soil quality, and fertilizer.

## Current State
- `Weather::agriculture_multiplier()` exists: Spring=1.2, Summer=1.0, Autumn=0.8, Winter=0.3.
- `ResourceType::FertileLand` exists in natural_resources.rs.
- No actual crop growth simulation.

## Definition of Done
- [ ] `growing_season_active` flag based on temperature and season.
- [ ] `crop_yield_modifier = rainfall_adequacy * temperature_suitability * soil_quality * fertilizer_bonus`.
- [ ] Rainfall adequacy: adequate(20-40in/yr)=1.0, excess(>40)=0.8, deficit(<20)=0.6, irrigated=min(1.0, supply/demand).
- [ ] Growing season affects food production in `ResourceBalance`.
- [ ] Frost events can destroy crops (Spring/Autumn frost risk).
- [ ] Irrigation infrastructure as a building that provides water to agricultural zones.

## Test Plan
- [ ] Unit test: growing season is active in Spring/Summer, inactive in Winter.
- [ ] Unit test: drought reduces crop yield by 40%.
- [ ] Integration test: agricultural zone produces food only during growing season.
- [ ] Integration test: irrigation prevents drought yield loss.

## Pitfalls
- Agricultural zones don't exist as a separate `ZoneType`; may use FertileLand resource deposits.
- Irrigation requires water supply infrastructure (WATER-001 dependency).
- Growing season length varies by climate zone (WEATHER-002 dependency).

## Code References
- `crates/simulation/src/weather.rs`: `Weather::agriculture_multiplier()`
- `crates/simulation/src/natural_resources.rs`: `ResourceType::FertileLand`
- Research: `environment_climate.md` section 4.3.4
