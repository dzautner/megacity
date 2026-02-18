# AUDIO-003: Traffic Audio System

**Category:** Audio / Spatial
**Priority:** T4
**Source:** sound_design.md -- Section 2.2

## Summary

Traffic audio proportional to TrafficGrid density. Three layers: engine hum (aggregate), tire noise, horns/brakes (event-based). Volume scales with traffic density per visible chunk. Time-of-day modulation (rush hour peaks). Road type affects sound character (highway = louder/lower, residential = quieter).

## Details

- Engine hum: continuous loop, volume = traffic_density * road_type_factor * time_mod
- Tire noise: mid-frequency whoosh on highway-speed roads
- Event sounds: horns at high congestion (>0.8 density), braking at intersections
- Highway roads: lower pitch, louder baseline, truck rumble component
- Residential streets: quieter, occasional individual car pass
- Time modulation: peak at 8-9am and 5-6pm (rush hour), minimum 2-5am

## Dependencies

- AUDIO-001 (audio bus hierarchy)
- Traffic system (TrafficGrid)

## Acceptance Criteria

- [ ] Traffic hum scales with density
- [ ] Road type affects sound character
- [ ] Rush hour peaks audible
- [ ] Horn/brake events at high congestion
