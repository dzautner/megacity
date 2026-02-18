# WATER-004: Stormwater Runoff Grid and Imperviousness Model

## Priority: T2 (Depth)

## Description
Implement a stormwater runoff system that calculates how much rainfall becomes surface runoff vs infiltration based on each cell's imperviousness. Impervious surfaces (roads, buildings, parking) generate runoff; pervious surfaces (grass, parks, forests) absorb water. This is the foundation for urban flooding.

## Current State
- Rain events exist in weather system but do not generate runoff.
- No imperviousness calculation per cell.
- No stormwater drainage infrastructure.
- No surface water accumulation.

## Definition of Done
- [ ] `imperviousness(cell)` function: road/building=0.95, parking=0.90, concrete=0.85, compacted soil=0.70, grass=0.35, forest=0.15, green roof=0.25, pervious pave=0.40.
- [ ] `runoff(cell) = rainfall_intensity * imperviousness * cell_area`.
- [ ] `infiltration(cell) = rainfall_intensity * (1 - imperviousness) * soil_permeability`.
- [ ] `StormwaterGrid` resource tracking accumulated runoff per cell.
- [ ] Runoff accumulates during rain events and drains via D8 flow to downstream cells.
- [ ] System runs conditionally only during rain/storm weather events.

## Test Plan
- [ ] Unit test: road cell produces 0.95 * rainfall as runoff.
- [ ] Unit test: forest cell produces 0.15 * rainfall as runoff.
- [ ] Integration test: heavy rain on a fully-paved area produces maximum runoff.
- [ ] Integration test: park absorbs most rainfall.

## Pitfalls
- Runoff calculation depends on D8 flow directions (share with POLL-005).
- Update frequency should match rainfall event duration, not every tick.
- Cell_area is implicit (CELL_SIZE^2 = 256 sq meters).

## Code References
- `crates/simulation/src/weather.rs`: `WeatherEvent::Rain`, `WeatherEvent::Storm`
- `crates/simulation/src/grid.rs`: `CellType`, cell land use
- Research: `environment_climate.md` sections 2.3.1-2.3.2
