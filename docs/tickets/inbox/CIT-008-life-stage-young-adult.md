# CIT-008: Life Stage -- YoungAdult (18-25) University and First Job

**Priority:** T2 (Depth)
**Complexity:** Medium (1-2 person-weeks)
**Dependencies:** CIT-002 (education levels), EDU-001 (education pipeline)
**Source:** social_agent_simulation.md Section 2.2

## Description

YoungAdult (18-25): university enrollment probability varies by family income (15% poverty to 90% wealthy). University graduation at 75% if enrolled. First job seeking at 16-25 with 80%+ probability when jobs available. High mobility (25% annual relocation). Marriage probability begins (2-6% per year). This is the most dynamic life stage with many branching outcomes.

## Definition of Done

- [ ] University enrollment probability based on family income class
- [ ] University attendance requires University building
- [ ] Graduation probability at 75% (modified by school quality, ambition)
- [ ] First job seeking with education-matched job requirements
- [ ] High annual relocation probability (25% base)
- [ ] Marriage probability starting at age 18 (2% rising to 6% by 25)
- [ ] Transition to Adult at age 26

## Test Plan

- Unit test: enrollment probability for each income class matches table
- Unit test: graduation probability correctly modified by quality and ambition
- Integration test: city with university sees higher education levels over time
- Integration test: young adults without university cap at HighSchool education

## Pitfalls

- University attendance conflicts with work schedule; students may work part-time
- First job for university student should be after graduation, not during

## Relevant Code

- `crates/simulation/src/citizen.rs` (LifeStage::YoungAdult, line 20)
- `crates/simulation/src/education_jobs.rs` (job_matching)
- `crates/simulation/src/life_simulation.rs` (education_advancement, life_events)
