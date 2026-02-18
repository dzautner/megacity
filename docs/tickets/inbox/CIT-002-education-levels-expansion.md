# CIT-002: Expand Education Levels from 0-3 to 0-5

**Priority:** T2 (Depth)
**Complexity:** Medium (2 person-weeks)
**Dependencies:** None
**Source:** social_agent_simulation.md Section 1.1 Education Level Distribution table

## Description

Current education is `u8` with values 0-3 (None, Elementary, HighSchool, University). Expand to `EducationLevel` enum with 6 values: None/SomeElementary (5%), ElementaryComplete (10%), HighSchool (35%), Bachelors (25%), Masters (15%), Doctorate (10%). Each level has a base salary multiplier (0.5x through 3.5x). This provides better granularity for job matching, income stratification, and education pipeline simulation.

## Definition of Done

- [ ] `EducationLevel` enum with 6 variants replaces raw u8
- [ ] `salary_multiplier()` method on enum returning 0.5, 0.7, 1.0, 1.8, 2.5, 3.5
- [ ] `base_salary_for_education` updated to use new enum
- [ ] Citizen spawner generates education with correct distribution
- [ ] Job matching system updated to use new education levels
- [ ] Existing education 0-3 mapped to new enum in save migration
- [ ] UI displays new education level names

## Test Plan

- Unit test: salary multipliers correct for each level
- Unit test: spawned education distribution matches target within 3%
- Integration test: job matching respects new education tiers
- Save migration test: old education 0-3 correctly maps to new enum

## Pitfalls

- Many systems reference `details.education` as u8; must update all call sites
- Job requirements across the codebase may assume 0-3 range

## Relevant Code

- `crates/simulation/src/citizen.rs` lines 90-114 (CitizenDetails, base_salary_for_education)
- `crates/simulation/src/education_jobs.rs` (job matching)
- `crates/simulation/src/life_simulation.rs` (education_advancement)
