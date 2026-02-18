# WATER-015: Groundwater Level and Quality Overlay

## Priority: T2 (Depth)

## Description
Implement a groundwater overlay showing both water level (availability) and quality (contamination) per cell. Helps players identify where wells will be productive, where contamination exists, and where depletion is occurring.

## Current State
- `GroundwaterGrid` has per-cell level and quality data.
- No dedicated groundwater overlay in the overlay system.
- `GroundwaterStats` shows city-wide averages but not per-cell.

## Definition of Done
- [ ] Groundwater level overlay: blue gradient from dry (red/empty) to saturated (deep blue).
- [ ] Groundwater quality overlay: green (clean) to brown (contaminated) gradient.
- [ ] Toggle between level and quality sub-overlays.
- [ ] Well locations shown as icons.
- [ ] Depletion warning: cells with level < 30 highlighted with warning color.
- [ ] Tooltip: level value, quality value, extraction rate, recharge rate.

## Test Plan
- [ ] Visual test: high groundwater areas show deep blue.
- [ ] Visual test: contaminated areas show brown.
- [ ] Visual test: wells shown as icons with status.

## Pitfalls
- Two sub-overlays (level and quality) need a clear toggle mechanism.
- Current u8 groundwater values may need mapping to meaningful display ranges.
- Well icons should be distinct from other service building icons.

## Code References
- `crates/rendering/src/overlay.rs`: overlay system
- `crates/simulation/src/groundwater.rs`: `GroundwaterGrid`
