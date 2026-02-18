# INFRA-150: Sky Dome and Cloud Shadows
**Priority:** T2
**Complexity:** M (1-3 days)
**Dependencies:** none
**Source:** master_architecture.md, Section 5.2 (Rendering Crate)

## Description
Add a sky dome with procedural sky colors (blue gradient, sunset/sunrise colors tied to day/night cycle). Cloud layer with shadows cast on terrain. Clouds as noise-based scrolling texture. Cloud density varies with weather (clear sky vs overcast from `weather.rs`). Cloud shadows move across terrain, adding visual life to the city.

## Definition of Done
- [ ] Sky dome mesh with procedural colors
- [ ] Sky color changes with time of day (blue day, orange sunset, dark night)
- [ ] Cloud layer with scrolling noise texture
- [ ] Cloud shadows projected onto terrain
- [ ] Cloud density tied to weather state
- [ ] Tests pass

## Test Plan
- Unit: Sky color at noon is blue; at sunset is orange
- Unit: Cloud density increases during rain weather
- Integration: Cloud shadows visibly move across the city

## Pitfalls
- Sky dome rendering must not interfere with UI overlays
- Cloud shadow requires a second render pass or projection
- Current `day_night.rs` handles lighting; coordinate with sky dome

## Relevant Code
- `crates/rendering/src/day_night.rs` -- day/night cycle
- `crates/rendering/src/terrain_render.rs` -- shadow receiving
