# SVC-003: Service Vehicle Dispatch System

**Priority:** T2 (Depth)
**Complexity:** High (4-5 person-weeks)
**Dependencies:** SVC-001 (hybrid coverage)
**Source:** historical_demographics_services.md Section 6.3, master_architecture.md Section 1.10

## Description

Spawn visible service vehicles (fire trucks, ambulances, police cars) that travel the road network to respond to incidents. Each service building has a vehicle pool. Dispatch selects nearest available vehicle. Vehicle pathfinds to incident, spends time at scene, returns to station. Response time = road distance / vehicle speed (accounting for traffic). Vehicles occupied at incidents reduce station capacity. Visual feedback: flashing lights, siren sound (placeholder). Only for dramatic events; background coverage uses grid-based model.

## Definition of Done

- [ ] `ServiceVehicle` component (vehicle_type, owning_station, status, target_cell)
- [ ] Vehicle pool per service building (based on capacity)
- [ ] Dispatch algorithm: nearest available vehicle to incident
- [ ] Vehicle pathfinding on road network (reuse citizen pathfinding)
- [ ] Response time calculation and tracking
- [ ] Vehicle at scene: occupies for resolution duration
- [ ] Vehicle return to station after resolution
- [ ] Occupied vehicles reduce station available capacity
- [ ] Visual rendering of service vehicles on roads
- [ ] Response time metrics per service type in stats panel

## Test Plan

- Unit test: nearest vehicle dispatched to incident
- Unit test: all vehicles occupied = no response (incident unresolved)
- Unit test: response time correct for road distance
- Integration test: visible fire trucks respond to fires
- Performance test: 20 simultaneous dispatches don't cause frame drops

## Pitfalls

- Vehicle pathfinding uses same system as citizens; at high citizen count may bottleneck
- Vehicles should have priority pathfinding (not wait in queue with citizen paths)

## Relevant Code

- `crates/simulation/src/services.rs` (ServiceBuilding)
- `crates/simulation/src/movement.rs` (pathfinding)
- `crates/simulation/src/fire.rs` (fire events)
- `crates/simulation/src/crime.rs` (crime events)
