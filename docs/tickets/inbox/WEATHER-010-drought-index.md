# WEATHER-010: Drought Index and Water Restriction Policies

## Priority: T2 (Depth)

## Description
Implement a drought index based on rolling 30-day rainfall average compared to expected rainfall. Drought severity tiers trigger water restrictions, agricultural yield reductions, wildfire risk multipliers, and reservoir level drops.

## Current State
- No precipitation tracking over time.
- No drought concept.
- No water restrictions.

## Definition of Done
- [ ] `drought_index = rolling_avg_rainfall(30_days) / expected_rainfall`.
- [ ] Normal (>0.8): no effects.
- [ ] Moderate drought (0.5-0.8): lawn watering banned (-20% demand), agriculture -30%, fire risk +100%.
- [ ] Severe drought (0.25-0.5): mandatory rationing (-40% demand, happiness -20%), agriculture -60%, fire risk +300%.
- [ ] Extreme drought (<0.25): emergency water imports, agriculture failure, fire risk +500%, wells drying up.
- [ ] `DroughtIndex` resource tracking current level and history.
- [ ] UI indicator showing drought severity.

## Test Plan
- [ ] Unit test: 15 days without rain in a zone expecting 30mm = drought_index < 0.5.
- [ ] Unit test: severe drought triggers mandatory rationing policy.
- [ ] Integration test: prolonged dry weather builds drought progressively.
- [ ] Integration test: drought + hot weather dramatically increases wildfire risk.

## Pitfalls
- Rainfall tracking requires accumulating precipitation data over time (not currently tracked).
- Expected rainfall varies by climate zone (WEATHER-002 dependency).
- Drought effects cascade across water, agriculture, fire, and happiness systems.

## Code References
- `crates/simulation/src/weather.rs`: `WeatherEvent::Rain`, precipitation
- `crates/simulation/src/forest_fire.rs`: fire risk
- Research: `environment_climate.md` section 4.4.4
