# CIT-089: Minimap

**Priority:** T4 (Polish)
**Complexity:** Low-Medium (1-2 person-weeks)
**Dependencies:** None
**Source:** master_architecture.md Section 1.18

## Description

Corner minimap showing full city overview with camera viewport indicator. Color-coded by zone type (green=residential, blue=commercial, yellow=industrial, gray=roads). Click on minimap to jump camera to location. Toggleable visibility. Minimap refreshes periodically (not every frame). Shows overlay colors when overlay is active.

## Definition of Done

- [ ] Minimap rendered in corner of screen
- [ ] Color-coded by zone type
- [ ] Camera viewport indicator rectangle
- [ ] Click to jump camera
- [ ] Toggle visibility (M key)
- [ ] Updates periodically (every 5 seconds)
- [ ] Reflects active overlay coloring

## Test Plan

- Unit test: minimap colors match zone types
- Unit test: click on minimap moves camera correctly
- Visual test: minimap accurately represents city layout

## Pitfalls

- Minimap rendering must be cheap (low-res, periodic update)
- Must handle zoom levels correctly

## Relevant Code

- `crates/rendering/src/camera.rs`
- `crates/ui/src/lib.rs`
