# EDU-008: School District Visualization and Assignment

**Priority:** T2 (Depth)
**Complexity:** Low (0.5 person-weeks)
**Dependencies:** EDU-001 (education pipeline)
**Source:** historical_demographics_services.md Section 3.4

## Description

Visualize school catchment areas as districts. Each school building has a catchment area based on road-network distance. Students assigned to nearest school within catchment. Catchment overlay shows which school serves each residential cell. Over-capacity schools have stressed (red) catchment. Under-served areas (no school in catchment) shown in gray. School quality displayed in overlay tooltip.

## Definition of Done

- [ ] School catchment area calculation (road-network BFS)
- [ ] Student assignment to nearest school
- [ ] Catchment overlay visualization
- [ ] Over-capacity visual indicator (red)
- [ ] Under-served area indicator (gray)
- [ ] School quality in tooltip
- [ ] Catchment updated when schools added/removed

## Test Plan

- Unit test: student assigned to nearest school
- Unit test: catchment area shown correctly
- Visual test: overlay shows expected patterns

## Pitfalls

- Catchment overlap resolution: assign to closest, not arbitrary

## Relevant Code

- `crates/simulation/src/education.rs`
- `crates/rendering/src/overlay.rs`
