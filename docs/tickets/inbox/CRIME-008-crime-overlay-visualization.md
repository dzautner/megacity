# CRIME-008: Enhanced Crime Overlay Visualization

**Priority:** T2 (Depth)
**Complexity:** Low (0.5 person-weeks)
**Dependencies:** CRIME-001 (crime types)
**Source:** social_agent_simulation.md Section 8, master_architecture.md Section 1.18

## Description

Enhance the crime overlay with crime type color-coding, hotspot highlighting, and trend arrows. Current overlay shows raw crime grid as single color ramp. New overlay: property crime (blue), violent crime (red), white collar (yellow), drug (green), organized (purple). Hotspot circles for areas > 2x city average. Trend arrows showing increasing/decreasing crime (requires historical tracking).

## Definition of Done

- [ ] Crime overlay color-coded by dominant crime type
- [ ] Hotspot circles for high-crime areas
- [ ] Historical crime data (last 12 months per chunk)
- [ ] Trend arrows (up/down/stable) per chunk
- [ ] Legend showing crime type colors
- [ ] Tooltip on hover showing crime rate and type breakdown

## Test Plan

- Visual test: different zones show different dominant crime types
- Visual test: hotspots visible in expected locations
- Visual test: trends reflect actual crime changes over time

## Pitfalls

- Historical tracking requires additional memory (12 months * chunk count)
- Color-coding for 5 crime types + intensity needs careful palette design

## Relevant Code

- `crates/rendering/src/overlay.rs` (OverlayState::Crime)
- `crates/simulation/src/crime.rs` (CrimeGrid)
