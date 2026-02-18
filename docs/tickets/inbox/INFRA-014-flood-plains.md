# INFRA-014: Flood Plain Detection and Risk Mapping
**Priority:** T2
**Complexity:** M (1-3 days)
**Dependencies:** INFRA-006, INFRA-012
**Source:** procedural_terrain.md, Section 7.4

## Description
Identify flood plain zones: low-lying flat areas adjacent to rivers and water bodies. Mark cells within a configurable elevation range above water level as flood risk zones. During heavy rain weather events, flood plains can flood, damaging buildings. Building on flood plains should reduce land value but provide fertile land bonus. Flood risk overlay in the overlay system.

## Definition of Done
- [ ] Flood plain cells identified (within 2m elevation of nearest water)
- [ ] Flood risk stored per cell
- [ ] Heavy rain events trigger flooding in flood plain cells
- [ ] Flooding damages buildings (condition reduction)
- [ ] Flood risk overlay mode added to `OverlayMode`
- [ ] Tests pass

## Test Plan
- Unit: Cells adjacent to rivers with low elevation delta are flagged as flood plain
- Unit: Flood damage applies only to flood plain cells during flood event

## Pitfalls
- Flood plain width depends on terrain slope -- flat areas have wider flood plains
- Need to interact with `weather.rs` rain events and `disasters.rs` flood events

## Relevant Code
- `crates/simulation/src/terrain.rs` -- flood plain detection
- `crates/simulation/src/disasters.rs` -- flood event handling
- `crates/simulation/src/weather.rs` -- rain intensity
- `crates/rendering/src/overlay.rs` -- new overlay mode
