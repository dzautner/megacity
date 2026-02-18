# CIT-046: Career Stage Progression System

**Priority:** T3 (Differentiation)
**Complexity:** Medium (1-2 person-weeks)
**Dependencies:** CIT-002 (education levels)
**Source:** social_agent_simulation.md Section 2.2

## Description

Career progression through 5 stages: EntryLevel, Junior (2-5 years), MidCareer (5-15), Senior (15-25), Executive (25+). Each stage has salary multiplier (1.0, 1.3, 1.8, 2.5, 3.5). Promotion probability per year: 15-20% for first promotions, 5-10% for senior, 2-5% for executive. Promotion modified by: education (+20% with matching degree), ambition personality, company (building) quality, city economic conditions. Career change: 3% annually for unhappy workers, resets to EntryLevel.

## Definition of Done

- [ ] `CareerStage` enum with 5 levels
- [ ] Years in stage tracking
- [ ] Salary multiplier per stage
- [ ] Annual promotion probability check
- [ ] Promotion modifiers: education, ambition, economy
- [ ] Career change mechanic (3% for unhappy)
- [ ] Career stage stored and serialized
- [ ] Career visible in citizen detail panel

## Test Plan

- Unit test: promotion probability correct per stage
- Unit test: salary multiplier applied correctly
- Unit test: career change resets to EntryLevel
- Integration test: experienced workforce has higher average salary

## Pitfalls

- Career stage is per-citizen state that must persist across save/load
- Executive tier should be rare (~5% of workforce)

## Relevant Code

- `crates/simulation/src/citizen.rs` (CitizenDetails.salary)
- `crates/simulation/src/education_jobs.rs`
