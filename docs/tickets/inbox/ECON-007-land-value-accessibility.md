# ECON-007: Land Value Accessibility Component
**Priority:** T2
**Complexity:** M
**Dependencies:** ECON-006
**Source:** economic_simulation.md, section 2.2

## Description
Add accessibility as a land value factor. Accessibility measures how easily a location can reach jobs, shopping, and services via the road/transit network. Uses network distance, not Euclidean.

- Job accessibility: weighted count of reachable jobs within 30-minute travel time
- Commercial accessibility: count of commercial buildings reachable within 15 minutes
- Transit accessibility: distance to nearest transit stop (when transit exists)
- Highway accessibility: distance to nearest highway ramp
- Accessibility computed via BFS on road network with travel time weights
- Update on slow tick (expensive computation)

## Definition of Done
- [ ] Job accessibility computed per cell via road network BFS
- [ ] Commercial accessibility computed similarly
- [ ] Transit/highway proximity measured
- [ ] Accessibility feeds into hedonic land value model
- [ ] Accessibility overlay available

## Test Plan
- Unit: Cell adjacent to highway has higher highway accessibility
- Unit: Cell in dense commercial area has higher commercial accessibility
- Integration: Build highway interchange, verify land value increase nearby

## Pitfalls
- BFS from every cell is O(cells^2) -- must sample or use landmark-based heuristic
- Travel time depends on traffic, which changes dynamically
- Cannot compute during heavy traffic without traffic-aware weights

## Relevant Code
- `crates/simulation/src/land_value.rs` -- accessibility integration
- `crates/simulation/src/road_graph_csr.rs` -- BFS/Dijkstra for travel time
- `crates/rendering/src/overlay.rs` -- accessibility overlay
