# SVC-010: Heating Service and Weather Integration

**Priority:** T2 (Depth)
**Complexity:** Low-Medium (1 person-week)
**Dependencies:** None
**Source:** historical_demographics_services.md, master_architecture.md

## Description

Heating service provides warmth during cold weather. HeatingBoiler: local, cheap, polluting. DistrictHeatingPlant: wider coverage, moderate cost, moderate pollution. GeothermalPlant: wide coverage, expensive, no pollution. Heating demand = max(0, 18 - temperature) * building_density. Buildings without heating coverage in cold weather: happiness -15, health -5, energy costs +50%. Current HeatingGrid exists; enhance with demand/supply.

## Definition of Done

- [ ] Heating demand calculated from weather temperature
- [ ] Heating supply from heating buildings in coverage area
- [ ] Supply < demand: partial heating, penalties proportional to deficit
- [ ] HeatingBoiler: cheap, polluting, local
- [ ] DistrictHeatingPlant: moderate, moderate
- [ ] GeothermalPlant: expensive, clean, wide
- [ ] No heating in cold weather: happiness -15, health -5
- [ ] Heating cost tracked in budget

## Test Plan

- Unit test: demand correctly calculated from temperature
- Unit test: heating coverage prevents cold penalties
- Unit test: boiler produces pollution
- Integration test: winter increases heating demand visibly

## Pitfalls

- Heating is irrelevant in warm climates; don't penalize tropical cities
- Current weather system may not have strongly cold temperatures

## Relevant Code

- `crates/simulation/src/heating.rs` (HeatingGrid, HeatingStats)
- `crates/simulation/src/weather.rs` (Weather.temperature)
