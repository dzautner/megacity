# POLL-006: Water Pollution Point Source Emissions by Building Type

## Priority: T2 (Depth)

## Description
Expand water pollution sources beyond just industrial buildings. The research doc defines 8 point source types (sewage outfall, treatment plants at various levels, industry, power plant cooling, mining) with distinct pollution rates and pollutant types.

## Current State
- Only industrial buildings near water cells generate water pollution.
- Intensity is a fixed formula: `10 + level * 7`.
- No sewage outfall concept.
- No treatment plant discharge modeling.
- No thermal pollution from power plant cooling.
- No pollutant type distinction (all pollution is generic).

## Definition of Done
- [ ] Point source table with 8+ source types and per-type emission rates.
- [ ] Sewage outfall: untreated = 80.0, primary = 32.0, secondary = 12.0, tertiary = 4.0.
- [ ] Heavy industry discharge = 50.0, light industry = 20.0.
- [ ] Power plant cooling water discharge = 15.0 (thermal pollution).
- [ ] Treatment effectiveness function: `treated_discharge(raw, level) = raw * (1 - effectiveness)`.
- [ ] Treatment levels: None=0%, Primary=60%, Secondary=85%, Tertiary=95%, Advanced=99%.
- [ ] Existing `SewagePlant` utility type connected to treatment level.

## Test Plan
- [ ] Unit test: `treatment_effectiveness` returns correct values for all levels.
- [ ] Unit test: tertiary treatment discharges 5% of raw pollution.
- [ ] Integration test: city without treatment plant has high water pollution near sewage outfall.
- [ ] Integration test: upgrading treatment level reduces downstream pollution.

## Pitfalls
- Sewage outfall location is not currently modeled (buildings don't have sewer connections).
- May need to place sewage discharge at the nearest water cell to the sewage treatment plant.
- Treatment level upgrade UI does not exist; needs building upgrade system.

## Code References
- `crates/simulation/src/water_pollution.rs`: `update_water_pollution`
- `crates/simulation/src/utilities.rs`: `UtilityType::SewagePlant`
- `crates/simulation/src/services.rs`: `ServiceType::WaterTreatmentPlant`
- Research: `environment_climate.md` sections 1.2.2, 1.2.4
