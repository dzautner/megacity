# DISASTER-011: Tornado Mitigation (Sirens, Shelters, Building Codes)

## Priority: T3 (Differentiation)

## Description
Implement tornado preparedness measures: tornado sirens, public storm shelters, residential safe rooms, wind-rated building codes, Doppler radar, underground utilities, and mobile home restrictions.

## Current State
- No tornado warning system.
- No shelter system.
- No tornado-specific building codes.

## Definition of Done
- [ ] Tornado sirens: $200K, 15-min warning, reduces deaths by 50%.
- [ ] Public storm shelters: $50K each, saves lives in EF3+.
- [ ] Residential safe rooms: $5K/home, saves occupants.
- [ ] Wind building codes: +1 EF level resistance, +10% build cost.
- [ ] Doppler radar: $1M, extends warning to 30 min.
- [ ] Underground utilities: 2x cost, reduces power outage duration.
- [ ] Mobile home restriction policy: bans mobile homes in tornado-prone zones.

## Test Plan
- [ ] Unit test: sirens reduce death rate by 50%.
- [ ] Unit test: wind building codes improve resilience by 1 EF level.
- [ ] Integration test: tornado warning allows citizens to reach shelters.

## Pitfalls
- Shelter coverage requires tracking which citizens can reach a shelter in warning time.
- Mobile homes don't exist as a building type yet.
- Warning time + shelter + distance calculation is complex.

## Code References
- Research: `environment_climate.md` section 5.4.4
