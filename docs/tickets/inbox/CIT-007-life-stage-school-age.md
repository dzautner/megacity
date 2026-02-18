# CIT-007: Life Stage -- SchoolAge (6-17) Behavior and Dropout

**Priority:** T2 (Depth)
**Complexity:** Medium (1-2 person-weeks)
**Dependencies:** EDU-001 (education pipeline)
**Source:** social_agent_simulation.md Section 2.1-2.2

## Description

SchoolAge citizens (6-17) attend school during weekdays. Dropout probability is 2-8% annually, modulated by family income, student happiness, school quality, and personality ambition. Dropout transitions student directly to workforce (unskilled). Elementary->HighSchool progression at age 14 with 92% base probability. School attendance requires school building within catchment radius.

## Definition of Done

- [ ] SchoolAge citizens commute to school on weekdays
- [ ] Dropout probability calculation with income/happiness/quality/ambition modifiers
- [ ] Dropout removes student from school, sets education to current level
- [ ] Elementary->HighSchool transition at age 14 with probability check
- [ ] Missing school building reduces education quality
- [ ] Dropout citizens enter workforce seeking unskilled jobs

## Test Plan

- Unit test: dropout probability matches table values for each modifier combination
- Unit test: student at poverty income has 4x dropout rate vs wealthy
- Integration test: city with no high school sees all students drop out at 14
- Integration test: dropout rate decreases when school quality improves

## Pitfalls

- Students who drop out need immediate job-seeking behavior
- School capacity limits may force some students to travel far or miss school

## Relevant Code

- `crates/simulation/src/citizen.rs` (LifeStage::SchoolAge, line 19)
- `crates/simulation/src/movement.rs` (CommutingToSchool/AtSchool states)
- `crates/simulation/src/life_simulation.rs` (education_advancement)
