# SERV-007: Park District System with Levels
**Priority:** T3
**Complexity:** L
**Dependencies:** ZONE-015
**Source:** cities_skylines_analysis.md, section 14.6

## Description
Implement park districts modeled after CS1's Parklife DLC. Players draw park boundaries, place park props/attractions, and parks level up based on visitor count and attractiveness.

- Park district: player-drawn boundary with entrance gate
- Park levels: L1 (local), L2 (small), L3 (city park), L4 (national park), L5 (star attraction)
- Level determined by: visitor count, number of props placed, variety of attractions
- Park entry fee: optional revenue source ($1-5 per visitor)
- Park effects: +land value in radius, +happiness for visitors, tourism attraction
- Park types: City Park, Amusement Park, Nature Reserve, Zoo
- Each type has unique props and level requirements

## Definition of Done
- [ ] Park district paintable with entrance
- [ ] Park leveling system based on visitors and props
- [ ] Entry fee revenue
- [ ] Land value and happiness bonuses
- [ ] Tourism attraction for high-level parks

## Test Plan
- Integration: Create park district, place props, verify leveling
- Integration: Set entry fee, verify park revenue in budget

## Pitfalls
- Park "props" need an asset system (benches, fountains, rides)
- Visitor tracking requires pathfinding citizens to parks
- Park boundary must not overlap other park boundaries

## Relevant Code
- `crates/simulation/src/districts.rs` -- park district type
- `crates/simulation/src/tourism.rs` -- park tourism contribution
- `crates/simulation/src/services.rs` -- park service buildings
