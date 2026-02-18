# CIT-018: Immigration Push-Pull Factor Model

**Priority:** T2 (Depth)
**Complexity:** Medium (1-2 person-weeks)
**Dependencies:** None
**Source:** social_agent_simulation.md Section 2.4, historical_demographics_services.md Section 2.4

## Description

Replace simple attractiveness score with a push-pull immigration model. Pull factors (attract immigrants): job availability (weight 0.3), housing availability (0.2), service quality (0.15), tax rate inversely (0.1), safety (0.1), education quality (0.1), environmental quality (0.05). City attractiveness = weighted sum of pull factors, 0-100 scale. Immigration rate = base_rate * (attractiveness / 50) * housing_vacancy_rate. Immigrants arrive in waves with demographic clustering (family chains, ethnic neighborhoods). Emigration rate = base_rate * (100 - attractiveness) / 50 * (1 - resilience).

## Definition of Done

- [ ] `PullFactors` resource with weighted components
- [ ] City attractiveness computed from pull factors
- [ ] Immigration rate formula with housing vacancy constraint
- [ ] Immigrant demographics skew young-adult (matching real patterns)
- [ ] Family chain immigration (immigrant attracts family members later)
- [ ] Emigration formula based on individual unhappiness vs city attractiveness
- [ ] ImmigrationStats tracks monthly in/out flows

## Test Plan

- Unit test: attractiveness formula produces 0-100 range
- Unit test: zero housing vacancy = zero immigration
- Unit test: high attractiveness = higher immigration rate
- Integration test: fast-growing city has positive net migration
- Integration test: declining city has negative net migration

## Pitfalls

- Immigration spikes can overwhelm housing capacity; need queueing
- Emigration of skilled workers creates brain drain feedback loop

## Relevant Code

- `crates/simulation/src/immigration.rs` (CityAttractiveness, immigration_wave)
- `crates/simulation/src/lifecycle.rs` (emigration)
