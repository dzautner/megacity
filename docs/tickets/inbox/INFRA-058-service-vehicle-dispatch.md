# INFRA-058: Service Vehicle Dispatch on Road Network
**Priority:** T2
**Complexity:** L (3-5 days)
**Dependencies:** INFRA-030
**Source:** master_architecture.md, M3; Section 6.5

## Description
Implement dispatch simulation for emergency services. Fire trucks, ambulances, and police cars travel on the road network from station to incident. Response time = pathfinding time on road network (not Euclidean distance). Response time affects outcome: fire damage, patient survival, crime resolution. Currently fire is coverage-based with no actual vehicle movement.

## Definition of Done
- [ ] Service vehicle entities that move on road network
- [ ] Dispatch: nearest available vehicle assigned to incident
- [ ] Response time computed from actual road network path
- [ ] Response time affects outcome (fire spread, patient health, arrest probability)
- [ ] Vehicle returns to station after incident
- [ ] Multiple simultaneous incidents handled (queue if all vehicles busy)
- [ ] Tests pass

## Test Plan
- Unit: Fire truck from station 50 cells away arrives in expected time based on road speed
- Unit: When all trucks busy, new incidents wait (fire spreads more)
- Integration: Fire response time visibly affects fire damage

## Pitfalls
- Traffic congestion delays emergency vehicles; may need priority lane or speed bonus
- Multiple vehicles from multiple stations dispatched to large incidents
- Vehicle entities need pathfinding which is already expensive

## Relevant Code
- `crates/simulation/src/fire.rs` -- fire response system
- `crates/simulation/src/services.rs` -- service building locations
- `crates/simulation/src/movement.rs` -- vehicle movement on roads
