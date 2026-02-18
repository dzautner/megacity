# POLL-020: Air Pollution Overlay Rendering with AQI Colors

## Priority: T1 (Core)

## Description
Update the air pollution overlay to use the 6-tier AQI color scheme (Green/Yellow/Orange/Red/Purple/Maroon) and display concentration values in the tooltip. Currently the overlay uses a simple gradient that doesn't communicate severity.

## Current State
- Pollution overlay exists in `overlay.rs`.
- Uses a basic color gradient based on u8 (0-255) values.
- No AQI color coding.
- No tooltip with concentration/AQI value.

## Definition of Done
- [ ] AQI color bands: 0-50=Green, 51-100=Yellow, 101-150=Orange, 151-200=Red, 201-300=Purple, 301+=Maroon.
- [ ] Color ramp updated to match EPA AQI standard colors.
- [ ] Tooltip shows: concentration value, AQI tier name, health advisory text.
- [ ] Handle both u8 (current) and f32 (future POLL-001) value ranges.
- [ ] Legend added to overlay showing tier colors and labels.

## Test Plan
- [ ] Visual test: overlay shows green for clean areas, red for polluted.
- [ ] Visual test: tooltip shows correct AQI tier label.
- [ ] Unit test: color mapping returns correct color for each tier boundary.

## Pitfalls
- Current u8 range maps differently than the 0-1000 f32 range from POLL-001.
- Must handle the transition gracefully (work with both ranges until migration).
- AQI color scheme assumes US EPA standards; other regions use different colors.

## Code References
- `crates/rendering/src/overlay.rs`: pollution overlay
- Research: `environment_climate.md` section 1.1.5
