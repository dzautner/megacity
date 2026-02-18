# CIT-082: Ferry Routes

**Priority:** T3 (Differentiation)
**Complexity:** Medium (3-4 person-weeks)
**Dependencies:** CIT-079 (transit template)
**Source:** master_architecture.md Section 1.7

## Description

Water-based transit connecting separated land areas. Ferry pier placement on coastline/river. Route drawn across water. Ferry vessels with set capacity and headway. Useful for cities with rivers/harbors where bridges are insufficient. Ferry slower than road but bypasses congestion. Tourism bonus for scenic routes.

## Definition of Done

- [ ] Ferry pier placement on water-adjacent cells
- [ ] Ferry route drawing across water
- [ ] Ferry vessel entities
- [ ] Capacity and headway settings
- [ ] Citizen mode choice includes ferry
- [ ] Tourism bonus for scenic routes
- [ ] Ferry revenue tracking

## Test Plan

- Unit test: ferry route crosses water
- Unit test: citizens choose ferry when faster than bridge detour
- Integration test: ferry connects separated neighborhoods

## Pitfalls

- Requires water bodies in map; useless on inland maps

## Relevant Code

- `crates/simulation/src/services.rs` (FerryPier)
