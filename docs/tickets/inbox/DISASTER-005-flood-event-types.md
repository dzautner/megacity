# DISASTER-005: Flood Event Types (Flash, River, Coastal, Dam Break)

## Priority: T2 (Depth)

## Description
Implement distinct flood event types with different sources, warning times, durations, and max depths. The research doc defines 6 flood types: flash flood, river flood, coastal surge, dam break, snowmelt flood, and urban flood.

## Current State
- `DisasterType::Flood` is a single type with radius 8 and elevation threshold 0.45.
- No distinction between flood sources.
- No warning time concept.
- No flood duration variation.

## Definition of Done
- [ ] `FloodType` enum: FlashFlood, RiverFlood, CoastalSurge, DamBreak, SnowmeltFlood, UrbanFlood.
- [ ] Flash flood: 0-1 hour warning, 2-6 hours duration, 2-6 ft max depth, triggered by intense localized rain.
- [ ] River flood: 12-48 hour warning, days-weeks duration, 4-20 ft depth, from sustained upstream rain.
- [ ] Coastal surge: 6-24 hour warning, 6-12 hours, 5-25 ft depth, from storms.
- [ ] Dam break: 0-2 hour warning, 2-8 hours, 10-40 ft depth, structural failure.
- [ ] Urban flood: 1-3 hour warning, 3-12 hours, 1-4 ft depth, overwhelmed drainage.
- [ ] Each type triggers differently and affects different map areas.
- [ ] Warning time allows evacuation (reduces casualties).

## Test Plan
- [ ] Unit test: flash flood has zero warning time.
- [ ] Unit test: river flood has 12-48 hour warning.
- [ ] Integration test: heavy sustained rain triggers river flood in low-lying areas.
- [ ] Integration test: warning time reduces casualties via evacuation.

## Pitfalls
- Must replace or heavily refactor existing `DisasterType::Flood`.
- Dam break requires dams to exist (POWER-007 dependency).
- Urban flood is distinct from drainage overflow (WATER-005); may merge or separate.

## Code References
- `crates/simulation/src/disasters.rs`: `DisasterType::Flood`
- Research: `environment_climate.md` sections 5.2.1-5.2.3
