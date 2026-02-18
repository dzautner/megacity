# TRAF-012: Train/Rail Transit System
**Priority:** T2
**Complexity:** XL
**Dependencies:** TRAF-005
**Source:** cities_skylines_analysis.md, section 11.1; master_architecture.md, section M3

## Description
Implement train system for long-distance passenger and cargo transit. Trains run on dedicated rail lines (separate from road network), serve large stations, and connect to outside world.

- Player places rail tracks (separate from road placement)
- Train stations: large footprint (6x6 cells minimum), high cost
- Passenger trains: 240 capacity, 120 km/h
- Cargo trains: freight movement alternative to trucks
- Train lines: player-defined routes between stations
- Cost: $2000/week per line + $800 per station
- Outside connection: trains connect to edge of map for import/export
- Train station boosts land value in large radius

## Definition of Done
- [ ] Rail track placement tool
- [ ] Train stations placeable
- [ ] Train lines with schedules
- [ ] Passenger and cargo service
- [ ] Outside connections for trade
- [ ] Station land value bonus

## Test Plan
- Integration: Build rail line between two stations, verify trains run
- Integration: Connect to map edge, verify import/export via train

## Pitfalls
- Rail network is completely separate from road network
- Train pathfinding on rail graph, not road graph
- Station footprint needs multi-cell building support (BLDG-002)
- Cargo trains interact with production chains (SERV-009)

## Relevant Code
- `crates/simulation/src/services.rs:ServiceType::TrainStation` -- already exists
- `crates/rendering/src/input.rs` -- rail placement tool
- `crates/simulation/src/outside_connections.rs` -- train connections
