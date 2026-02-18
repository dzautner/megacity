# INFRA-100: Controller Support
**Priority:** T4
**Complexity:** M (1-3 days)
**Dependencies:** none
**Source:** master_architecture.md, M5

## Description
Add gamepad/controller support for playing on couch or Steam Deck. Left stick: camera pan. Right stick: camera rotate. Triggers: zoom in/out. Face buttons: select tool, confirm, cancel. D-pad: cycle through tools. Radial menu for tool selection. Cursor mode: virtual cursor controlled by right stick for precise placement.

## Definition of Done
- [ ] Camera control with analog sticks
- [ ] Tool selection via gamepad buttons
- [ ] Radial menu for tool categories
- [ ] Virtual cursor for precise placement
- [ ] All core gameplay accessible via controller
- [ ] Tests pass

## Test Plan
- Unit: Analog stick input translates to smooth camera movement
- Integration: Complete game loop using controller only

## Pitfalls
- Precision placement (road curves) is challenging with controller; cursor acceleration helps
- Button count is limited; radial menus help but add interaction steps
- Steam Deck compatibility requires Steam Input integration

## Relevant Code
- `crates/rendering/src/input.rs` -- input handling
- `crates/rendering/src/camera.rs` -- camera controls
