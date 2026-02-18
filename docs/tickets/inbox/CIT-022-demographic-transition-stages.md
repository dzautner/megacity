# CIT-022: Demographic Transition Stages

**Priority:** T2 (Depth)
**Complexity:** Medium (2 person-weeks)
**Dependencies:** CIT-021 (population pyramid), CIT-012 (birth rate)
**Source:** historical_demographics_services.md Section 2.3

## Description

Track city through 5 demographic transition stages as it grows: Stage 1 (Pre-transition: high birth/death), Stage 2 (Early growth: death falls, pop explodes), Stage 3 (Late growth: birth falls), Stage 4 (Stability: low birth/death), Stage 5 (Decline: birth below replacement). Each stage modifies birth rates, death rates, and immigration patterns. City transitions between stages based on healthcare quality, education level, and income.

## Definition of Done

- [ ] `DemographicStage` enum with 5 variants
- [ ] Transition conditions based on healthcare quality, education, income
- [ ] Birth rate modifier per stage
- [ ] Death rate modifier per stage
- [ ] Stage displayed in city stats
- [ ] Event journal entry on stage transition

## Test Plan

- Unit test: new city starts at Stage 2
- Unit test: high healthcare quality advances to Stage 3
- Integration test: natural city progression through stages over 100+ game years

## Pitfalls

- Stage transitions should be gradual, not sudden jumps
- Stage 5 (decline) should be avoidable with good immigration policy

## Relevant Code

- `crates/simulation/src/lifecycle.rs` (birth/death rates)
- `crates/simulation/src/immigration.rs` (immigration patterns)
