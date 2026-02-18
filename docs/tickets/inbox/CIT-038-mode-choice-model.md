# CIT-038: Mode Choice Model (Car vs Transit vs Walk)

**Priority:** T2 (Depth)
**Complexity:** Medium (3-4 person-weeks)
**Dependencies:** Public transit system (not yet ticketed)
**Source:** social_agent_simulation.md Section 4.4, master_architecture.md Section 1.6

## Description

Citizens choose transportation mode using utility-based comparison. Walk: free, max 2km, speed 5km/h. Car: fuel cost, parking cost, speed from road network, always available. Transit: fare cost, wait time + ride time, requires transit stop within 500m. Utility per mode = -alpha*cost - beta*time - gamma*waiting. Mode with highest utility (least negative) chosen. Car ownership based on income and parking availability. Transit ridership metric tracks system effectiveness.

## Definition of Done

- [ ] `mode_utility()` function for walk, car, transit
- [ ] Walk: time = distance / 5km/h, cost = 0, max 2km
- [ ] Car: time from road network with traffic, cost = fuel + parking
- [ ] Transit: time = walk_to_stop + wait + ride + walk_from_stop, cost = fare
- [ ] Mode with highest utility selected
- [ ] Car ownership probability by income class
- [ ] Transit ridership tracking
- [ ] Mode share statistics (% car / transit / walk)

## Test Plan

- Unit test: short distance prefers walking
- Unit test: car faster but more expensive than transit
- Unit test: good transit attracts riders from car
- Integration test: mode share shifts with transit investment

## Pitfalls

- Transit system doesn't exist yet; placeholder with bus stop walking time
- Car parking availability not yet modeled; use simplified cost

## Relevant Code

- `crates/simulation/src/movement.rs` (pathfinding, citizen movement)
- `crates/simulation/src/traffic.rs`
