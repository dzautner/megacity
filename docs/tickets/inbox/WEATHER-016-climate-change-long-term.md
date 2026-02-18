# WEATHER-016: Long-Term Climate Change from CO2 Emissions

## Priority: T3 (Differentiation)

## Description
Track cumulative CO2 emissions from fossil fuel power plants and industry. After threshold, trigger long-term climate effects: +1-3F temperature increase, more extreme weather events, sea level rise (edge flooding on coastal maps), and longer droughts.

## Current State
- No CO2 emission tracking.
- No climate change mechanic.
- No sea level concept.

## Definition of Done
- [ ] `CumulativeCO2` resource tracking total city CO2 emissions since game start.
- [ ] CO2 per source: coal=1.0 ton/MWh, gas=0.4, oil=0.8, biomass=0.0 (net).
- [ ] Climate threshold: 1M tons = +1F avg temp, 5M = +2F, 20M = +3F.
- [ ] More extreme weather: disaster frequency +10% per 1F increase.
- [ ] Sea level rise: at +3F, lowest-elevation water-adjacent cells flood permanently.
- [ ] `EnvironmentalScore` metric: aggregate measure of city environmental impact.
- [ ] Yearly assessment system.

## Test Plan
- [ ] Unit test: coal plant generates 1.0 ton CO2 per MWh.
- [ ] Unit test: crossing 1M ton threshold increases temperature by 1F.
- [ ] Integration test: city powered entirely by coal sees temperature rise over game-years.
- [ ] Integration test: city powered by renewables has zero CO2 growth.

## Pitfalls
- Climate change effects are very long-term; may not be noticeable in short games.
- Sea level rise permanently altering terrain is a major visual/gameplay change.
- Players may not connect power choices to climate outcomes without clear UI.

## Code References
- `crates/simulation/src/weather.rs`: temperature modification
- `crates/simulation/src/grid.rs`: `Cell.elevation`, `CellType::Water`
- Research: `environment_climate.md` section 7.4 (Eco game reference)
