# INFRA-052: Park-and-Ride Facilities
**Priority:** T3
**Complexity:** M (1-3 days)
**Dependencies:** INFRA-037, INFRA-049
**Source:** transportation_simulation.md, Section 6.5

## Description
Implement park-and-ride as placeable building at transit stations. Size: 250-500 spaces (BRT), 500-2000 (commuter rail), 1000-5000 (major rail hub). Catchment: 5-8 km radius. Citizens compare driving all the way (parking + gas + congestion) vs driving to P&R + transit fare. Each P&R user removes one car from congested portion of network. Mode choice: drive-to-P&R as a sub-mode of transit.

## Definition of Done
- [ ] Park-and-ride building type placeable at transit stations
- [ ] Capacity in parking spaces
- [ ] Catchment radius for mode choice eligibility
- [ ] Mode choice includes drive-to-P&R option
- [ ] Traffic reduction from P&R usage tracked
- [ ] Tests pass

## Test Plan
- Unit: 500-space P&R at 80% = 400 fewer cars downtown
- Integration: P&R at suburban rail station shows ridership

## Pitfalls
- P&R only effective if remaining transit trip is frequent and fast
- P&R lot itself generates traffic on surrounding roads
- Must be at transit stop, not standalone

## Relevant Code
- `crates/simulation/src/buildings.rs` -- building type
- `crates/simulation/src/movement.rs` -- drive-to-P&R mode
