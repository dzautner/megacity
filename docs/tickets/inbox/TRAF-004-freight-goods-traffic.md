# TRAF-004: Freight/Goods Traffic on Road Network
**Priority:** T2
**Complexity:** L
**Dependencies:** none
**Source:** cities_skylines_analysis.md, section 7; master_architecture.md, section 6.3

## Description
Add freight/goods movement on the road network. Currently all traffic is commuter traffic. Industrial and commercial buildings should generate truck traffic for goods delivery.

- Industrial buildings generate outbound freight (production -> commercial/export)
- Commercial buildings generate inbound freight (receiving goods for sale)
- Freight vehicles are larger (truck equivalence factor 2.0-3.0 vs car=1.0)
- Trucks use pathfinding on road network, prefer highways
- Truck traffic contributes to traffic density, road wear, noise, pollution
- Heavy traffic ban policy blocks trucks from specific districts

## Definition of Done
- [ ] Industrial buildings generate freight trips
- [ ] Commercial buildings receive freight trips
- [ ] Truck traffic visible on road network
- [ ] Trucks contribute to congestion with vehicle equivalence factor
- [ ] Trucks increase road wear and noise
- [ ] Heavy traffic ban policy affects truck routing

## Test Plan
- Integration: Build industrial + commercial, verify truck traffic appears on connecting roads
- Integration: Apply heavy traffic ban, verify trucks reroute

## Pitfalls
- Freight trip generation rate needs tuning (too many trucks = traffic death)
- Truck routing should prefer highways over local roads (add preference weight)
- Freight not serialized currently (regenerate on load?)

## Relevant Code
- `crates/simulation/src/production.rs` -- freight generation
- `crates/simulation/src/movement.rs` -- freight vehicle movement
- `crates/simulation/src/traffic.rs` -- truck equivalence factor
- `crates/simulation/src/road_maintenance.rs` -- truck wear factor
