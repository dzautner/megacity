# CIT-039: Homelessness Pipeline Enhancement

**Priority:** T2 (Depth)
**Complexity:** Medium (1-2 person-weeks)
**Dependencies:** CIT-004 (household)
**Source:** social_agent_simulation.md Section 11, historical_demographics_services.md Section 5.4

## Description

Enhance existing homelessness system with causal pipeline: job_loss -> savings_depletion -> rent_default -> eviction -> homelessness. Homeless rate = f(housing_cost, income, mental_health, substance_abuse). Shelters provide temporary housing (90-day limit). Permanent supportive housing provides long-term solution. Homeless citizens have: reduced health (-20), increased crime risk (2x), reduced happiness (-30 already exists), visible on streets. Recovery path: shelter -> job -> savings -> housing.

## Definition of Done

- [ ] Causal pipeline: job loss -> savings depletion -> eviction -> homeless
- [ ] Eviction when rent_burden > 0.5 for 3+ months AND savings = 0
- [ ] Shelter capacity tracking
- [ ] Shelter time limit (90 game-days)
- [ ] Permanent supportive housing building type
- [ ] Recovery path with measurable progress
- [ ] Homeless count metric in city stats
- [ ] Homeless concentration in specific areas (downtown, parks)

## Test Plan

- Unit test: job loss + no savings leads to homelessness after delay
- Unit test: shelter prevents worst health outcomes
- Unit test: permanent housing enables recovery
- Integration test: economic downturn increases homelessness visibly

## Pitfalls

- Current Homeless component exists; extend rather than replace
- Homelessness must be recoverable (not permanent state)

## Relevant Code

- `crates/simulation/src/homelessness.rs` (Homeless component, shelter-seeking)
- `crates/simulation/src/welfare.rs` (WelfareStats)
