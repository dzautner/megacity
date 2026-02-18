# INFRA-097: Interchange Templates and Roundabout Builder
**Priority:** T4
**Complexity:** L (3-5 days)
**Dependencies:** none
**Source:** master_architecture.md, M5

## Description
Add pre-built interchange templates and a roundabout builder. Templates: diamond interchange, cloverleaf, trumpet, diverging diamond. Roundabout: circular road segment with yield entries, configurable radius (2-6 cells). Roundabouts have lower delay than signals at moderate volumes (INFRA-033) but higher land usage. Template placement snaps to existing highway/arterial intersections.

## Definition of Done
- [ ] Roundabout placement tool with configurable radius
- [ ] At least 3 interchange templates
- [ ] Template snaps to existing road intersections
- [ ] Roundabout intersection model in pathfinding (yield-based delay)
- [ ] Tests pass

## Test Plan
- Unit: Roundabout creates circular road segment with correct connections
- Unit: Interchange template connects to existing roads
- Integration: Roundabout reduces intersection delay at moderate volumes

## Pitfalls
- Roundabout geometry with Bezier curves is complex
- Interchange templates require significant space (8x8+ cells)
- Multi-level interchanges need elevation support (bridges/tunnels)

## Relevant Code
- `crates/simulation/src/road_segments.rs` -- Bezier road placement
- `crates/rendering/src/input.rs` -- template placement tool
