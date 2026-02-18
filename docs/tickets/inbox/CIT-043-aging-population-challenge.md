# CIT-043: Aging Population Challenge

**Priority:** T3 (Differentiation)
**Complexity:** Medium (2 person-weeks)
**Dependencies:** CIT-021 (population pyramid), CIT-011 (retired stage)
**Source:** historical_demographics_services.md Section 2.3, master_architecture.md Section 1.20

## Description

Late-game challenge: when dependency ratio (non-working / working) exceeds 0.65, city faces aging crisis. Effects: pension costs rise, healthcare demand spikes (5x for elderly), workforce shrinks, tax base declines, nursing home demand increases. Solutions: pro-natalist policies, immigration to bring young workers, automation (reduces workforce need), pension reform. Dependency ratio displayed prominently in stats.

## Definition of Done

- [ ] Dependency ratio computed: (0-14 + 65+) / (15-64)
- [ ] Warning at dependency ratio > 0.50
- [ ] Crisis at dependency ratio > 0.65
- [ ] Pension cost = retired_count * pension_rate
- [ ] Healthcare demand multiplier for elderly population
- [ ] Workforce shortage reduces economic output
- [ ] Pro-natalist policy option (childcare subsidies)
- [ ] Immigration policy to attract young workers

## Test Plan

- Unit test: dependency ratio correctly calculated
- Unit test: high ratio increases pension costs
- Integration test: city that doesn't attract immigrants ages into crisis

## Pitfalls

- Aging crisis should be a gradual pressure, not sudden failure
- Japan-style demographic decline is a real-world reference

## Relevant Code

- `crates/simulation/src/lifecycle.rs`
- `crates/simulation/src/economy.rs`
