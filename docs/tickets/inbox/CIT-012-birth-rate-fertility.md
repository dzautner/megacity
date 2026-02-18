# CIT-012: Birth Rate and Fertility Model

**Priority:** T2 (Depth)
**Complexity:** Medium (1-2 person-weeks)
**Dependencies:** CIT-004 (household), CIT-009 (adult life stage)
**Source:** social_agent_simulation.md Section 2.3, historical_demographics_services.md Section 2.2

## Description

Implement age-specific fertility rates. Peak fertility at 25-29 (112 per 1000 women/year), declining through age bands. Total Fertility Rate (TFR) sum determines population replacement. Urban TFR typically 1.5-2.0 (below replacement 2.1). Birth probability modified by: income (U-shaped), education (negative), housing quality, childcare availability, existing children. Births create new Child citizen entities linked to parent household.

## Definition of Done

- [ ] Age-specific fertility rates: 15-19 (22/1000), 20-24 (66), 25-29 (112), 30-34 (95), 35-39 (48), 40-44 (10), 45-49 (1)
- [ ] TFR tracked as city statistic
- [ ] Birth probability modifiers: income, education, housing quality, childcare, existing children
- [ ] New citizen created as Child entity with parent/household links
- [ ] Birth registered in event journal
- [ ] Population pyramid updated with new births

## Test Plan

- Unit test: fertility rate at peak age band matches target
- Unit test: high education reduces birth probability
- Unit test: childcare availability increases birth probability
- Integration test: city TFR settles in 1.5-2.0 range
- Integration test: city without childcare has lower TFR

## Pitfalls

- New child entities increase citizen count; must respect LOD and virtual population caps
- Both parents must be in the city (partner not emigrated)

## Relevant Code

- `crates/simulation/src/life_simulation.rs` (life_events, children spawning)
- `crates/simulation/src/citizen_spawner.rs` (spawn logic)
- `crates/simulation/src/citizen.rs` (Family component, line 282)
