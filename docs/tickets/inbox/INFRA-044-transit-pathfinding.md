# INFRA-044: Multi-Modal Transit Pathfinding
**Priority:** T2
**Complexity:** L (3-5 days)
**Dependencies:** INFRA-037, INFRA-028
**Source:** transportation_simulation.md, Section 4.1 (Step 4)

## Description
Implement multi-modal pathfinding for transit trips: walk to stop -> ride route -> transfer -> ride route -> walk to destination. Build a transit network graph with three link types: walking links (between stops and buildings, with walk speed cost), waiting links (at each stop, cost = headway/2), in-vehicle links (along route, cost = ride time). Find shortest path in this multi-modal graph using A*. Load passengers onto route segments; track boardings/alightings per stop.

## Definition of Done
- [ ] Multi-modal graph with walk, wait, and ride links
- [ ] A* pathfinding on multi-modal graph
- [ ] Walking links from building to nearest stops (within catchment)
- [ ] Wait time = headway/2 at each stop
- [ ] Passenger loading on route segments
- [ ] Boarding/alighting counts per stop
- [ ] Tests pass

## Test Plan
- Unit: Trip from building 100m from bus stop: walk + wait + ride
- Unit: Trip requiring transfer: walk + wait + ride + transfer wait + ride + walk
- Integration: Transit ridership numbers are plausible for city size

## Pitfalls
- Multi-modal graph is larger than road graph; performance-critical
- Transfer links between routes at shared stops need careful modeling
- Wait time varies by headway; frequent routes are strongly preferred

## Relevant Code
- `crates/simulation/src/road_graph_csr.rs` -- CSR graph pattern to extend
- `crates/simulation/src/movement.rs` -- citizen pathfinding
