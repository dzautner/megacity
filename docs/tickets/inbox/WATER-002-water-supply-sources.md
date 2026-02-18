# WATER-002: Water Supply Source Buildings (Wells, Intake, Desal)

## Priority: T1 (Core)

## Description
Implement the water supply source hierarchy: groundwater wells, surface water intake, reservoir, and desalination plant. Each source type has different capacity, cost, water quality, and environmental impact.

## Current State
- `ServiceType::WellPump` exists in the services system and draws down groundwater.
- No surface water intake building.
- No reservoir building.
- No desalination plant.
- No capacity tracking per source.

## Definition of Done
- [ ] `WaterSource` component with `source_type`, `capacity_mgd`, `quality`, `operating_cost`.
- [ ] Well pump: 0.5 MGD, low cost, depends on groundwater level, groundwater quality.
- [ ] Surface water intake: 5 MGD, moderate cost, must be placed on water cell, quality varies.
- [ ] Reservoir: 20 MGD, high cost, stores 90 days supply, large footprint (8x8).
- [ ] Desalination: 10 MGD, very high cost, consistent quality, placed on coast.
- [ ] Each source contributes to `WaterSupply.total_supply_mgd`.
- [ ] Sources degrade when polluted: quality affects treatment cost.

## Test Plan
- [ ] Unit test: well capacity = 0.5 MGD per well.
- [ ] Unit test: reservoir stores 90-day buffer at full capacity.
- [ ] Integration test: placing a well increases total water supply.
- [ ] Integration test: well near polluted groundwater provides low-quality water.

## Pitfalls
- Surface water intake must be placed on CellType::Water cells; need placement validation.
- Desalination requires coastal access (map-dependent).
- Reservoir footprint (8x8) may conflict with other buildings.

## Code References
- `crates/simulation/src/groundwater.rs`: well pump interaction
- `crates/simulation/src/services.rs`: `ServiceType::WellPump`
- Research: `environment_climate.md` section 2.2
