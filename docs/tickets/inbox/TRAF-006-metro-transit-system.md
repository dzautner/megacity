# TRAF-006: Metro/Subway Transit System
**Priority:** T2
**Complexity:** XL
**Dependencies:** TRAF-005
**Source:** cities_skylines_analysis.md, section 11.1; master_architecture.md, section M3

## Description
Implement metro/subway system. Underground rail with high capacity (180 passengers/train), immune to surface traffic. Metro stations boost land value. This is the workhorse transit for large cities.

- Player places metro stations (underground)
- Player draws metro lines connecting stations
- Trains run between stations at 80 km/h
- Station construction cost: $500/week + building cost
- Line cost: $1,200/week
- Metro stations boost land value +15-25 in radius
- Citizens consider metro: walk_to_station + wait + ride + walk_from_station
- High capacity makes metro efficient for dense corridors

## Definition of Done
- [ ] Metro stations placeable (underground visualization)
- [ ] Metro lines connectable between stations
- [ ] Trains run on schedule
- [ ] Citizens use metro when time-competitive with car
- [ ] Land value boost around stations
- [ ] Metro costs tracked in budget

## Test Plan
- Integration: Build metro line, verify ridership from adjacent residents
- Integration: Verify land value increase near metro stations

## Pitfalls
- Underground rendering layer needed (separate from surface)
- Metro is NOT on road network -- separate graph for transit
- Station placement must avoid water/existing underground infrastructure
- Metro lines crossing creates transfer stations (mode interchange)

## Relevant Code
- `crates/simulation/src/movement.rs` -- multi-modal pathfinding
- `crates/simulation/src/services.rs` -- metro station as service
- `crates/simulation/src/land_value.rs` -- station proximity bonus
- `crates/rendering/src/overlay.rs` -- underground transit view
