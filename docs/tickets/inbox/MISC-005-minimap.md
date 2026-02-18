# MISC-005: Minimap
**Priority:** T4
**Complexity:** M
**Dependencies:** none
**Source:** master_architecture.md, section M5

## Description
Add a minimap in the corner of the screen showing the full city at a glance. Minimap shows zone colors, roads, and current camera viewport. Click on minimap to jump to location.

- Minimap renders simplified top-down view of grid
- Color by zone type: green (residential), blue (commercial), yellow (industrial), gray (office)
- Roads shown as white/gray lines
- Current viewport shown as rectangle overlay
- Click to center camera on clicked location
- Toggle minimap visibility
- Optional: show overlay data on minimap (traffic, pollution, land value)

## Definition of Done
- [ ] Minimap renders in corner
- [ ] Zone colors correct
- [ ] Roads visible
- [ ] Viewport rectangle shown
- [ ] Click to navigate

## Test Plan
- Integration: Click on minimap, verify camera moves to location

## Pitfalls
- Minimap rendering must be efficient (not re-render full grid every frame)
- Render to texture at lower resolution, update periodically
- UI layering: minimap must not interfere with other panels

## Relevant Code
- `crates/ui/src/lib.rs` -- minimap UI component
- `crates/rendering/src/camera.rs` -- viewport position for minimap
