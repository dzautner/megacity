# CIT-005: Founding Population Generator

**Priority:** T2 (Depth)
**Complexity:** Low (1 person-week)
**Dependencies:** CIT-001 (demographics), CIT-002 (education levels)
**Source:** social_agent_simulation.md Section 1.3

## Description

When a new city starts, generate a plausible founding demographic instead of random noise. Founding populations skew young-adult (pioneers, young families): 18-25 (25%), 26-35 (35%), 36-45 (20%), 46-55 (10%), 56-65 (7%), 66-80 (3%). Education skews practical: HighSchool (40%), Bachelors (30%), Elementary (15%), Masters (10%), Doctorate (5%). As the city matures, the age pyramid naturally evolves through births, aging, death, immigration, and emigration.

## Definition of Done

- [ ] `generate_founding_population()` function with weighted age distribution
- [ ] Education distribution for founding population matches practical skew
- [ ] Called during new game initialization
- [ ] Produces gender-balanced population
- [ ] Initial households formed from founding population
- [ ] Age pyramid visible in UI shows young-adult skew

## Test Plan

- Unit test: 1000 founding citizens, median age 28-32
- Unit test: education distribution within 5% of targets
- Unit test: gender split within 48-52%
- Visual test: age pyramid UI shows expected shape

## Pitfalls

- Must coordinate with existing citizen_spawner to avoid double-spawning
- Founding population must have enough working-age adults to fill initial jobs

## Relevant Code

- `crates/simulation/src/citizen_spawner.rs` (spawn logic)
- `crates/app/src/main.rs` (game initialization)
