# INFRA-035: One-Way Street Directionality
**Priority:** T2
**Complexity:** M (1-3 days)
**Dependencies:** none
**Source:** transportation_simulation.md, Section 3.5

## Description
Enhance one-way road support. Currently `RoadType::OneWay` exists but direction is implicit. Add explicit direction to one-way road segments. One-way streets double capacity in the favored direction (both lanes same way). Support one-way pair configuration: two parallel one-way streets, one block apart. Trade-offs: higher throughput but increased travel distance (avg 1/2 block), slight land value reduction, pathfinding must respect directionality (already handled by directed CSR edges).

## Definition of Done
- [ ] One-way road segments have explicit direction (forward/backward along Bezier)
- [ ] CSR graph creates directed edges for one-way roads
- [ ] One-way capacity doubled vs equivalent two-way
- [ ] Visual arrow indicator on one-way road rendering
- [ ] Land value slight reduction near one-way streets
- [ ] Tests pass

## Test Plan
- Unit: Pathfinding cannot route against one-way direction
- Unit: One-way road has double per-direction capacity vs two-way
- Integration: Player can create one-way pairs downtown

## Pitfalls
- Converting existing two-way to one-way must update CSR graph edges
- One-way roads at map edges can create dead-end traps
- Bus routes on one-way streets must split onto paired streets

## Relevant Code
- `crates/simulation/src/road_segments.rs` -- direction on segments
- `crates/simulation/src/road_graph_csr.rs` -- directed edge creation
- `crates/rendering/src/road_render.rs` -- directional arrow overlay
