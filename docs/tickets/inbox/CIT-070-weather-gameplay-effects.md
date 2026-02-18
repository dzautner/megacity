# CIT-070: Weather Gameplay Effects

**Priority:** T2 (Depth)
**Complexity:** Medium (2-3 person-weeks)
**Dependencies:** None
**Source:** master_architecture.md Section 1.11

## Description

Weather currently exists but is largely cosmetic. Add real gameplay effects. Cold (<0C): heating demand, pipe freeze risk, road salt cost, slower construction. Hot (>35C): cooling demand, heat stroke risk (elderly), water demand increase. Rain: reduced outdoor activity, traffic speed -10%, construction delay, stormwater. Snow: road clearing cost, traffic speed -30%, school closures. Storm: damage risk, flight cancellations, event cancellations.

## Definition of Done

- [ ] Cold effects: heating demand, pipe risk, road cost
- [ ] Hot effects: cooling demand, health risk, water demand
- [ ] Rain effects: outdoor activity, traffic, construction
- [ ] Snow effects: road clearing, traffic, school closure
- [ ] Storm effects: damage, travel disruption
- [ ] Weather modifiers applied to relevant systems
- [ ] Weather alerts for extreme conditions
- [ ] Seasonal construction efficiency

## Test Plan

- Unit test: cold weather increases heating demand
- Unit test: rain reduces traffic speed by 10%
- Unit test: snow reduces traffic speed by 30%
- Integration test: weather visibly affects city operations

## Pitfalls

- Weather effects should be manageable, not devastating (except extreme events)
- Must tune so weather doesn't make the game annoying

## Relevant Code

- `crates/simulation/src/weather.rs` (Weather, update_weather)
- `crates/simulation/src/heating.rs`
- `crates/simulation/src/traffic.rs`
