# MISC-008: Accessibility (Colorblind Modes, Key Remapping)
**Priority:** T4
**Complexity:** M
**Dependencies:** none
**Source:** master_architecture.md, section M5

## Description
Add accessibility features: colorblind overlay modes, key remapping, UI scaling, and screen reader text. Widens the audience and is increasingly expected in modern games.

- Colorblind modes: Protanopia, Deuteranopia, Tritanopia filter on overlays
- Zone colors adjusted per colorblind mode (use shapes in addition to colors)
- Key remapping: all keyboard shortcuts configurable
- UI scale: 80% to 150% scaling for readability
- High contrast mode for overlays

## Definition of Done
- [ ] 3 colorblind modes available
- [ ] Key remapping in settings
- [ ] UI scale configurable
- [ ] High contrast overlay option

## Test Plan
- Visual: Overlays distinguishable in each colorblind mode
- Integration: Remap key, verify new binding works

## Pitfalls
- Colorblind simulation requires post-processing shader or palette swap
- Bevy's input system supports rebinding but may need wrapper
- UI scaling affects layout calculations

## Relevant Code
- `crates/rendering/src/overlay.rs` -- colorblind palette variants
- `crates/rendering/src/input.rs` -- key remapping
- `crates/ui/src/theme.rs` -- UI scaling
