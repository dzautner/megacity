# WEATHER-018: Enhanced Seasonal Rendering Effects

## Priority: T2 (Depth)

## Description
Enhance the existing seasonal rendering with more visual effects. Currently grass color changes by season. Add: falling leaves in autumn, snow on buildings in winter, flowers in spring, sun glare in summer, rain streaks during rain events, and storm clouds during storms.

## Current State
- `Season::grass_color()` changes terrain tint by season.
- Day/night cycle exists in `day_night.rs`.
- No precipitation particles.
- No seasonal building decorations.
- No weather particle effects.

## Definition of Done
- [ ] Autumn: falling leaf particles near tree cells.
- [ ] Winter: snow accumulation on building roofs (white tint on top faces).
- [ ] Spring: flower particles near park cells, brighter lighting.
- [ ] Summer: heat shimmer effect on hot days, longer shadows.
- [ ] Rain: rain streak particles during Rain/Storm events.
- [ ] Storm: darker sky, lightning flash effect.
- [ ] Snow: snowflake particles during winter precipitation.
- [ ] All effects toggleable for performance.

## Test Plan
- [ ] Visual test: autumn shows falling leaves near trees.
- [ ] Visual test: winter shows snow on buildings.
- [ ] Visual test: rain particles visible during Rain event.
- [ ] Performance test: particle effects do not drop FPS below 30.

## Pitfalls
- Particle effects can be expensive; must LOD by camera distance.
- Must not obscure gameplay-critical information.
- Shader effects (heat shimmer, lightning) require GPU capabilities.

## Code References
- `crates/rendering/src/day_night.rs`: time-of-day rendering
- `crates/rendering/src/terrain_render.rs`: terrain color
- `crates/simulation/src/weather.rs`: `Season::grass_color()`
