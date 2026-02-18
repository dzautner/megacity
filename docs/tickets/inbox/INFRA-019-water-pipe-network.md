# INFRA-019: Water Pipe Network (Hybrid Auto-Along-Road + Manual Trunk Mains)
**Priority:** T2
**Complexity:** XL (1-2 weeks)
**Dependencies:** none
**Source:** underground_infrastructure.md, Water Pipe Network section

## Description
Implement a hybrid water distribution system. Distribution pipes auto-extend along roads (no player action needed) using BFS from connected trunk mains. Trunk mains are manually placed underground infrastructure connecting water sources (treatment plants, reservoirs) to the distribution network. Water flows through the pipe network with pressure simulation: pressure decreases with distance from pumping station and elevation gain. Buildings not connected to the pipe network lack water service.

## Definition of Done
- [ ] `WaterPipeNetwork` resource with trunk mains and distribution grid
- [ ] Distribution pipes auto-extend along roads from trunk main connection points
- [ ] Pressure simulation: pressure = source_pressure - friction_loss - elevation_head
- [ ] Buildings check water connection for service flag
- [ ] Water pressure overlay mode
- [ ] Tests pass

## Test Plan
- Unit: Building adjacent to road connected to trunk main has water
- Unit: Pressure drops with distance from pump station
- Integration: Disconnecting trunk main cuts water to downstream buildings

## Pitfalls
- Current `utilities.rs` uses BFS flood-fill for water; need migration path
- Pressure calculation needs elevation data from terrain (INFRA-001 dependency soft)
- Auto-along-road means road demolition must update pipe network

## Relevant Code
- `crates/simulation/src/utilities.rs` -- current BFS water coverage
- `crates/simulation/src/grid.rs` -- road cells, building locations
