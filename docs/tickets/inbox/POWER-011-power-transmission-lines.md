# POWER-011: Power Line Transmission and Service Radius

## Priority: T1 (Core)

## Description
Implement power line placement that connects generators to consumers. Buildings need to be within POWER_RANGE=6 cells of a power line to receive service. Power lines follow roads automatically in dense areas, and high-voltage transmission lines connect distant generators.

## Current State
- No power line infrastructure.
- No power service radius.
- Power is implicitly available everywhere.

## Definition of Done
- [ ] Power lines: auto-follow roads (underground in dense areas).
- [ ] High-voltage transmission lines: separate placeable infrastructure for distant generators.
- [ ] Transformer substations: required every 20 cells of transmission line.
- [ ] Service radius: POWER_RANGE = 6 cells from a power line.
- [ ] Transmission losses: 2% per 10 cells of distance from generator to consumer.
- [ ] `has_power` flag on buildings based on connectivity to a generator.
- [ ] Buildings without power: no function, happiness penalty, may trigger abandonment.

## Test Plan
- [ ] Unit test: building 4 cells from power line has power.
- [ ] Unit test: building 8 cells from power line does NOT have power.
- [ ] Unit test: transmission loss at 20 cells = 4%.
- [ ] Integration test: building a power line to an area provides power service.
- [ ] Integration test: distant generator loses efficiency through transmission.

## Pitfalls
- Auto-following roads means road changes affect power service (must re-evaluate).
- Transformer substation requirement every 20 cells may be too granular for players.
- May want to simplify to "roads carry power" for first pass.

## Code References
- `crates/simulation/src/services.rs`: service coverage pattern
- Research: `environment_climate.md` section 3.5.4
