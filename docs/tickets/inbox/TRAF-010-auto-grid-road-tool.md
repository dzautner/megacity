# TRAF-010: Auto-Grid Road Placement Tool
**Priority:** T3
**Complexity:** M
**Dependencies:** none
**Source:** urban_planning_zoning.md, section 4.1

## Description
Implement an auto-grid road placement tool. Player defines a rectangular area and block size, and the tool auto-generates a grid street pattern. This is faster than manually placing each road.

- Player draws rectangle (start corner, end corner)
- Configurable block size (4-8 cells, default 6)
- Configurable road type (Local, Avenue, Boulevard)
- Tool generates horizontal and vertical roads within rectangle
- Preview overlay before confirmation
- Cost calculated and displayed before placement

## Definition of Done
- [ ] Auto-grid tool in road placement toolbar
- [ ] Configurable block size and road type
- [ ] Preview overlay shows grid before placement
- [ ] All roads placed on confirmation with cost deduction

## Test Plan
- Integration: Use auto-grid tool, verify correct grid pattern created
- Integration: Verify block size parameter affects grid density

## Pitfalls
- Must handle terrain obstacles (water, existing buildings)
- Grid must align with existing roads where possible
- Cost preview must account for all road cells

## Relevant Code
- `crates/rendering/src/input.rs` -- auto-grid tool
- `crates/simulation/src/grid.rs:WorldGrid` -- grid modification
- `crates/simulation/src/road_segments.rs` -- segment creation for grid roads
