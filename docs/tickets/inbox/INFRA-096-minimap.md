# INFRA-096: Minimap
**Priority:** T4
**Complexity:** M (1-3 days)
**Dependencies:** none
**Source:** master_architecture.md, M5

## Description
Add a minimap in corner of screen showing the full city at a glance. Color-coded by zone type (green=residential, blue=commercial, yellow=industrial). Camera viewport rectangle shown on minimap. Click on minimap to move camera. Minimap updates periodically (not every frame). Toggle minimap visibility.

## Definition of Done
- [ ] Minimap rendered as UI element in screen corner
- [ ] Zone-type color coding
- [ ] Camera viewport indicator
- [ ] Click-to-navigate on minimap
- [ ] Toggleable visibility
- [ ] Tests pass

## Test Plan
- Unit: Minimap colors match zone types
- Unit: Clicking minimap moves camera to correct position
- Integration: Minimap provides useful navigation aid

## Pitfalls
- Minimap rendering should not impact game performance
- Minimap size relative to screen resolution needs to be responsive
- Overlay modes could also show on minimap (traffic heatmap, etc.)

## Relevant Code
- `crates/ui/src/lib.rs` -- UI rendering
- `crates/simulation/src/grid.rs` -- zone type data
