# CIT-036: Housing Choice Utility Model

**Priority:** T3 (Differentiation)
**Complexity:** Medium (2-3 person-weeks)
**Dependencies:** CIT-004 (household), CIT-001 (demographics)
**Source:** social_agent_simulation.md Section 4.2

## Description

Citizens choose housing using utility maximization. Housing utility = affordability_weight * rent_affordability + commute_weight * commute_quality + school_weight * school_quality + safety_weight * safety + environment_weight * environmental_quality. Weights vary by household type: families weight schools 3x, singles weight commute 2x, retirees weight safety 2x. Citizens periodically evaluate and relocate if utility of alternative > current + switching_cost.

## Definition of Done

- [ ] `housing_utility()` function with weighted factors
- [ ] Household-type-specific weights
- [ ] Periodic utility evaluation (every 30 game-days)
- [ ] Relocation when alternative utility exceeds current by >20%
- [ ] Switching cost (moving expenses, emotional cost)
- [ ] Housing search targets best available within budget
- [ ] Relocation updates home building occupancy

## Test Plan

- Unit test: family with children weights schools heavily
- Unit test: single young adult weights commute heavily
- Unit test: relocation only when utility difference exceeds switching cost
- Integration test: families cluster near schools, workers near jobs

## Pitfalls

- Utility computation for all citizens every 30 days is expensive; use staggered evaluation
- Housing availability constrains choice; best utility option may be full

## Relevant Code

- `crates/simulation/src/life_simulation.rs`
- `crates/simulation/src/citizen.rs`
