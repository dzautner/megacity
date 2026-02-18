# DISASTER-021: Emergency Shelter System for Displaced Citizens

## Priority: T2 (Depth)

## Description
Implement emergency shelters for citizens displaced by disasters. Displaced citizens need temporary housing or they leave the city. Public buildings (schools, community centers) can serve as emergency shelters. Purpose-built shelters house more people.

## Current State
- Destroyed buildings despawn citizens.
- No displaced citizen tracking.
- No shelter concept.

## Definition of Done
- [ ] `DisplacedCitizen` tracking: citizens from destroyed/damaged buildings become displaced.
- [ ] Emergency shelters: schools, libraries, community centers can be designated as shelters.
- [ ] Purpose-built shelter: 2x2 building, 500 person capacity, $200K.
- [ ] Displaced citizens without shelter: leave city within 7 game-days.
- [ ] Shelter capacity limits: exceeded = overflow citizens leave.
- [ ] Recovery: displaced citizens return to rebuilt homes.
- [ ] Happiness penalty for displaced citizens (-20).
- [ ] Dashboard: displaced count, shelter capacity, expected departures.

## Test Plan
- [ ] Unit test: destroyed building creates N displaced citizens.
- [ ] Unit test: shelter absorbs displaced citizens up to capacity.
- [ ] Integration test: citizens leave city if no shelter available.
- [ ] Integration test: citizens return after homes rebuilt.

## Pitfalls
- Tracking individual displaced citizens is expensive; may use aggregate counts.
- Rebuilding homes requires the repair/rebuild system (DISASTER-014).
- Citizens leaving during a disaster cascade into population loss.

## Code References
- `crates/simulation/src/disasters.rs`: disaster damage
- `crates/simulation/src/immigration.rs`: citizen departure
- Research: `environment_climate.md` section 5.7
