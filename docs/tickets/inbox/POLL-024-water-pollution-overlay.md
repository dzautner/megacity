# POLL-024: Water Pollution Overlay with Quality Tiers

## Priority: T2 (Depth)

## Description
Update the water pollution overlay to show 6-tier water quality classification with color coding. Water cells should visually change color based on pollution level in the main terrain rendering (not just the overlay).

## Current State
- Water pollution overlay exists in `overlay.rs`.
- No tier-based coloring.
- Water cells always render the same blue regardless of pollution.

## Definition of Done
- [ ] Overlay color tiers: Pristine=deep blue, Clean=light blue, Moderate=green-blue, Polluted=yellow-brown, Heavy=brown, Toxic=dark brown/black.
- [ ] Tooltip: pollution level, tier name, health effects, water usability.
- [ ] Terrain rendering: water cell color tint shifts from blue (clean) to brown (polluted).
- [ ] Visual feedback: sewage outfall discharge visible as discolored water.

## Test Plan
- [ ] Visual test: clean water shows blue, polluted water shows brown.
- [ ] Visual test: tooltip displays correct tier for each pollution level.
- [ ] Visual test: terrain water color matches overlay indication.

## Pitfalls
- Terrain water color change requires modifying `terrain_render.rs`.
- Must handle the transition between overlay-on and overlay-off consistently.
- Water pollution grid uses u8 currently; tier boundaries need mapping.

## Code References
- `crates/rendering/src/overlay.rs`: water pollution overlay
- `crates/rendering/src/terrain_render.rs`: water cell rendering
- Research: `environment_climate.md` section 1.2.5
