# POLL-003: Air Pollution Health Effects with AQI Tiers

## Priority: T2 (Depth)

## Description
Implement the 6-tier AQI-equivalent health effect system for air pollution. Currently pollution affects land value and happiness indirectly, but there is no direct health_rate modifier based on concentration tiers. The research doc defines specific dose-response thresholds from "Good" (0-50, health_rate +0.01) to "Hazardous" (701-1000, health_rate -0.20).

## Current State
- `pollution.rs` does not directly modify citizen health.
- `groundwater.rs` and `water_pollution.rs` have health penalty systems, but air pollution does not.
- Land value penalty from pollution exists in `land_value.rs` but is basic.
- No AQI overlay label system.

## Definition of Done
- [ ] `air_pollution_health_modifier(concentration: f32) -> f32` function implemented per research doc.
- [ ] System that applies health rate modifier to citizens based on their home cell's air pollution level.
- [ ] Land value multiplier: `(1.0 - concentration / 2000.0).max(0.5)`.
- [ ] Happiness penalty: `-0.1` per 100 units of concentration.
- [ ] Immigration penalty: cities with avg AQI > 200 see 30% reduced immigration rate.
- [ ] Tourism penalty: pollution > 150 in tourist areas reduces tourism by 40%.
- [ ] AQI label shown in overlay tooltip (Good/Moderate/Unhealthy-SG/Unhealthy/Very Unhealthy/Hazardous).

## Test Plan
- [ ] Unit test: health modifier returns correct value for each concentration tier boundary.
- [ ] Unit test: land value multiplier clamps at 0.5 for max pollution.
- [ ] Integration test: citizens in highly polluted area lose health over time.
- [ ] Integration test: immigration drops when average city pollution is high.

## Pitfalls
- Depends on POLL-001 (wider pollution range) being implemented first, or this must work with the current u8 range temporarily.
- Health effects should not stack with water pollution and groundwater penalties in a way that makes citizens die instantly.

## Code References
- `crates/simulation/src/pollution.rs`: `PollutionGrid`
- `crates/simulation/src/happiness.rs`: `update_happiness`
- `crates/simulation/src/immigration.rs`: `CityAttractiveness`
- `crates/simulation/src/tourism.rs`: tourism systems
- Research: `environment_climate.md` sections 1.1.5, 1.1.6
