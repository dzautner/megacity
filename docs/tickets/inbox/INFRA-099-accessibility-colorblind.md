# INFRA-099: Accessibility (Colorblind Modes, Key Remapping)
**Priority:** T4
**Complexity:** M (1-3 days)
**Dependencies:** none
**Source:** master_architecture.md, M5

## Description
Add accessibility features: colorblind mode (deuteranopia, protanopia, tritanopia) with alternative color palettes for overlays and zone colors. Full key remapping for all keyboard shortcuts. UI scaling option. High contrast mode for text. Screen reader support for key information (population, budget).

## Definition of Done
- [ ] 3 colorblind mode options with alternative palettes
- [ ] Key remapping UI for all shortcuts
- [ ] UI scale slider (75%-200%)
- [ ] High contrast text mode
- [ ] Accessibility settings menu
- [ ] Tests pass

## Test Plan
- Unit: Colorblind palette has sufficient contrast between all zone types
- Unit: Remapped keys function correctly
- Integration: Game is playable with each colorblind mode enabled

## Pitfalls
- Zone colors must remain distinguishable in all colorblind modes
- Overlay heatmaps need alternative color ramps (not red-green)
- Key remapping must handle conflicts (two actions on same key)

## Relevant Code
- `crates/rendering/src/overlay.rs` -- color palettes
- `crates/rendering/src/input.rs` -- key bindings
- `crates/ui/src/lib.rs` -- UI scaling
