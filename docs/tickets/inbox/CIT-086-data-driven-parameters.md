# CIT-086: Data-Driven Game Parameters

**Priority:** T2 (Depth)
**Complexity:** High (4-6 person-weeks)
**Dependencies:** None
**Source:** master_architecture.md Section 1.16, T2

## Description

Extract all hardcoded game parameters into external data files (RON format). Categories: building stats (capacity, cost, construction time), road parameters (speed, lanes, width), service radii (per service type per tier), economy constants (tax rates, salary multipliers, cost of living), policy effects (modifiers per policy), zone demand weights, happiness weights, crime factors. Override hierarchy: base game data -> mod data -> user data. This is the foundation for moddability.

## Definition of Done

- [ ] RON data files for each parameter category
- [ ] Data loading at game start
- [ ] Hot-reload support for development
- [ ] Override hierarchy for modding
- [ ] At least 200 extracted parameters
- [ ] Documentation of all parameters
- [ ] Default values match current hardcoded values
- [ ] Validation of loaded data (range checks)

## Test Plan

- Unit test: loaded parameters match expected defaults
- Unit test: override file replaces base values
- Unit test: invalid parameter rejected with error
- Integration test: game runs identically with extracted parameters

## Pitfalls

- Massive refactor touching every simulation file
- Must not break existing game behavior during extraction

## Relevant Code

- Every file in `crates/simulation/src/` contains hardcoded constants
- `crates/simulation/src/config.rs` (existing constants)
