# POWER-007: Hydroelectric Dam Power Plant

## Priority: T2 (Depth)

## Description
Implement hydroelectric dam as a renewable, dispatchable power source. Must be placed across a river (water cells). Output depends on water flow which varies seasonally. Provides flood control as a co-benefit.

## Current State
- No power generation buildings exist.
- Water cells exist in terrain.

## Definition of Done
- [ ] Hydro dam: 200 MW capacity, 0.40 average capacity factor.
- [ ] Must be placed on water cells (placement validation).
- [ ] Seasonal variation: Spring=0.50 (snowmelt), Summer=0.30 (low flow), Autumn=0.35, Winter=0.45.
- [ ] Rain events boost output by 20%.
- [ ] Fuel cost: $0/MWh.
- [ ] Construction cost: $1B, build time: 15 game-days.
- [ ] 6x3 building footprint across water cells.
- [ ] Flood control: reduces downstream flood risk by 50%.
- [ ] Reservoir creation: cells upstream of dam become water cells.

## Test Plan
- [ ] Unit test: hydro output varies by season.
- [ ] Unit test: rain event increases output.
- [ ] Integration test: dam placed on water cells generates power.
- [ ] Integration test: dam reduces downstream flooding during heavy rain.

## Pitfalls
- Dam placement requires a specific terrain configuration (crossing water cells).
- Reservoir creation changes the map terrain permanently.
- Flood control interaction with WATER-005 needs careful design.

## Code References
- `crates/simulation/src/grid.rs`: `CellType::Water`
- Research: `environment_climate.md` section 3.2
