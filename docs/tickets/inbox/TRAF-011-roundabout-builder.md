# TRAF-011: Roundabout Builder Tool
**Priority:** T4
**Complexity:** M
**Dependencies:** none
**Source:** master_architecture.md, section T4; cities_skylines_analysis.md, section 17.8

## Description
Automated roundabout creation tool. Player clicks intersection, tool generates circular road with proper entry/exit connections. Roundabouts are the most efficient intersection type but tedious to build manually.

- Player clicks center point and specifies radius (2-5 cells)
- Tool generates circular road around center
- Connects existing approach roads to roundabout
- Road type: one-way (counterclockwise/clockwise depending on driving side)
- Roundabout traffic logic: yield on entry, priority on roundabout
- Small roundabout: 2-cell radius, 3-4 approaches
- Large roundabout: 4-5 cell radius, 4-6 approaches

## Definition of Done
- [ ] Roundabout tool places circular one-way road
- [ ] Existing approach roads connected automatically
- [ ] Traffic yields on entry to roundabout
- [ ] Configurable radius

## Test Plan
- Integration: Place roundabout at 4-way intersection, verify traffic flows
- Integration: Verify roundabout handles more traffic than signalized intersection

## Pitfalls
- Circular road on square grid creates jagged edges -- use Bezier curves
- Connection to existing roads needs careful segment merging
- Roundabout traffic logic distinct from signalized intersection

## Relevant Code
- `crates/rendering/src/input.rs` -- roundabout tool
- `crates/simulation/src/road_segments.rs` -- curved segment creation
- `crates/simulation/src/road_graph_csr.rs` -- roundabout intersection model
