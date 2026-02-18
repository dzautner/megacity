# SVC-025: Power Generation Types and Characteristics

**Priority:** T2 (Depth)
**Complexity:** Medium (2 person-weeks)
**Dependencies:** SVC-023 (power grid)
**Source:** master_architecture.md Section 1.13

## Description

Each power generation type has unique characteristics. Coal: 500MW, cheap, high pollution, reliable. Gas: 200MW, moderate cost/pollution, fast ramp. Nuclear: 1000MW, no pollution, expensive, public fear. Solar: 50MW, no pollution, weather-dependent, daytime only. Wind: 100MW, no pollution, wind-dependent, intermittent. Hydro: 200MW, no pollution, requires river. Geothermal: 50MW, no pollution, location-specific. Energy storage (batteries) smooths renewable intermittency.

## Definition of Done

- [ ] Power plant types with capacity, cost, pollution, reliability
- [ ] Coal: high capacity, high pollution
- [ ] Gas: moderate, fast ramp-up
- [ ] Nuclear: high capacity, no pollution, expensive, NIMBY factor
- [ ] Solar: weather-dependent output, daytime only
- [ ] Wind: wind-speed-dependent output
- [ ] Hydro: requires water proximity
- [ ] Battery storage to buffer intermittent renewables
- [ ] Generation mix visible in power grid panel

## Test Plan

- Unit test: coal plant produces pollution
- Unit test: solar output zero at night
- Unit test: wind output proportional to wind speed
- Integration test: renewable-only grid with storage maintains power

## Pitfalls

- Carbon tax policy should make clean energy economically competitive
- Nuclear NIMBY should reduce nearby land value

## Relevant Code

- `crates/simulation/src/utilities.rs` (UtilityType: PowerPlant, SolarPanel, WindTurbine)
