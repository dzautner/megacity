# SVC-024: Water/Sewage Pipe Network

**Priority:** T2 (Depth)
**Complexity:** High (5-7 person-weeks)
**Dependencies:** None
**Source:** master_architecture.md Section 1.12

## Description

Replace binary has_water BFS with pipe network. Pipes auto-follow roads by default (CS2 style) but with capacity limits. Water tower/pump station provides water pressure. Pipe capacity determines max flow. When demand > pipe capacity, pressure drops (buildings at end of line lose water). Sewage network separate: sewage flows from buildings to treatment plant. Treatment plant capacity limits. Pipe aging: old pipes leak (water loss) and break (service interruption).

## Definition of Done

- [ ] Pipes auto-follow road placement
- [ ] Water pressure calculation from source
- [ ] Pipe capacity limits
- [ ] Pressure drop at end of line
- [ ] Sewage network (parallel to water)
- [ ] Treatment plant capacity
- [ ] Pipe age tracking and leak probability
- [ ] Pipe break events with repair mechanic
- [ ] Underground view/overlay for pipe network

## Test Plan

- Unit test: pipe following road creates coverage
- Unit test: pressure drops with distance
- Unit test: pipe break causes local water loss
- Integration test: growing city outgrows pipe capacity

## Pitfalls

- Full pressure-flow simulation is complex; use simplified capacity model
- Pipe placement UX should be automatic (not manual pipe drawing)

## Relevant Code

- `crates/simulation/src/utilities.rs` (UtilitySource, propagate_utilities)
- `crates/simulation/src/grid.rs` (Cell.has_water)
