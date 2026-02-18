# SERV-002: Service Vehicle Dispatch on Road Network
**Priority:** T2
**Complexity:** XL
**Dependencies:** SERV-001
**Source:** master_architecture.md, section M3; cities_skylines_analysis.md, section 8

## Description
Service vehicles (fire trucks, ambulances, police cars, garbage trucks) should travel on the road network to reach emergencies and service points. Response time depends on distance and traffic conditions.

- Fire trucks dispatched to fire locations, travel on road network, response time matters
- Ambulances dispatched to sick citizens, travel to hospital
- Police cars patrol districts, respond to crime events
- Garbage trucks follow collection routes through districts
- Response time = pathfinding distance / speed * traffic_factor
- Faster response = better outcome (less fire damage, lives saved)
- Vehicle count limited by service building capacity

## Definition of Done
- [ ] Fire trucks dispatch and travel on road network
- [ ] Ambulances dispatch to sick citizens
- [ ] Response time computed from path distance and traffic
- [ ] Response time affects outcome (fire damage, survival rate)
- [ ] Service vehicles visible on roads

## Test Plan
- Integration: Start fire far from station, verify slower response and more damage
- Integration: Add fire station closer to fire-prone area, verify faster response

## Pitfalls
- Vehicle pathfinding is expensive -- limit concurrent dispatches
- Service vehicles need priority routing (emergency vehicles bypass traffic in some cities)
- Garbage routes are periodic, not event-driven (different from emergency dispatch)

## Relevant Code
- `crates/simulation/src/fire.rs` -- fire truck dispatch
- `crates/simulation/src/services.rs` -- dispatch logic
- `crates/simulation/src/movement.rs` -- vehicle movement on road network
- `crates/simulation/src/road_graph_csr.rs` -- pathfinding for dispatch
