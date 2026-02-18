# TRAF-007: Citizen Mode Choice (Car/Transit/Walk/Bike)
**Priority:** T2
**Complexity:** L
**Dependencies:** TRAF-005
**Source:** cities_skylines_analysis.md, section 11.3; master_architecture.md, section M3

## Description
Implement citizen mode choice where citizens evaluate multiple transportation options and pick the fastest/cheapest. Currently all citizens drive. Mode choice is the core mechanic that makes transit investment worthwhile.

- Modes: car, bus, metro, walking, bicycle
- Each mode has: access_time, wait_time, travel_time, comfort_penalty
- Car: drive_time (traffic-aware) * 1.0
- Bus: walk_to_stop + wait(frequency) + ride_time + walk_from_stop, comfort * 0.8
- Metro: walk_to_station + wait(frequency) + ride_time + walk_from_station, comfort * 0.9
- Walking: distance / walk_speed (5 km/h), max ~2km practical
- Bicycle: distance / bike_speed (15 km/h), requires bike infrastructure, max ~5km
- Multi-modal: Walk -> Bus -> Metro -> Walk (transfer penalty ~3 min each)
- Citizen chooses mode with lowest perceived_time = total_time / comfort_factor

## Definition of Done
- [ ] Citizens evaluate all available modes
- [ ] Mode with lowest perceived time selected
- [ ] Multi-modal trips supported (bus -> metro)
- [ ] Transfer penalties applied at mode changes
- [ ] Mode split visible in transportation panel (% by mode)

## Test Plan
- Unit: Walking 200m is preferred over driving 200m (parking overhead)
- Unit: Metro for 5km trip preferred over bus in traffic
- Integration: Build metro, verify car mode share decreases

## Pitfalls
- Evaluating all mode combinations per citizen per trip is expensive -- cache/precompute
- Citizens must have access to mode (near bus stop, near metro station, own car)
- Income affects car ownership (low-income may not have car option)

## Relevant Code
- `crates/simulation/src/movement.rs` -- mode choice before pathfinding
- `crates/simulation/src/citizen.rs:Citizen` -- current_mode field
- `crates/ui/src/info_panel.rs` -- mode split display
