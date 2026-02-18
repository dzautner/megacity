# INFRA-138: Era Progression with Visual and Mechanical Changes
**Priority:** T3
**Complexity:** L (3-5 days)
**Dependencies:** INFRA-055
**Source:** master_architecture.md, M4

## Description
Implement era progression where city advances through historical periods with visual and mechanical changes. Example eras: Early Settlement (dirt roads, basic services), Industrial (factories, streetcars), Modern (highways, suburbs), Contemporary (transit, mixed-use, sustainability). Each era unlocks new building types, road types, and policies. Building meshes change style per era. Transition triggered by population + time milestones.

## Definition of Done
- [ ] Era enum with at least 4 eras
- [ ] Progression triggers (population + time)
- [ ] Per-era building mesh styles
- [ ] Per-era road type and technology unlocks
- [ ] Per-era policy options
- [ ] Era transition notification
- [ ] Tests pass

## Test Plan
- Unit: Reaching 50K pop + 20 game-years triggers Industrial era
- Unit: Industrial era unlocks factory buildings
- Integration: City visually changes as eras progress

## Pitfalls
- Era transitions must not invalidate existing buildings
- Mixing eras (historic district in modern city) needs handling
- Building style changes across the whole city simultaneously may look odd

## Relevant Code
- `crates/simulation/src/unlocks.rs` -- progression system
- `crates/rendering/src/building_meshes.rs` -- era-specific meshes
