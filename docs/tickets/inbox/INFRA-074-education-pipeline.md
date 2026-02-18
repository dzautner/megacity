# INFRA-074: Education Pipeline with Graduation Rates
**Priority:** T2
**Complexity:** M (1-3 days)
**Dependencies:** INFRA-057
**Source:** master_architecture.md, M3

## Description
Implement education pipeline: elementary (6-12), high school (13-18), university (19-22+). Each level requires school capacity. Students enrolled based on school proximity and capacity. Graduation rate depends on school quality (funding level). Graduated citizens have higher skill level -> better jobs -> higher income -> higher property tax. Dropout rate from overcrowded or underfunded schools.

## Definition of Done
- [ ] Education levels: Elementary, HighSchool, University
- [ ] Student enrollment by age and school proximity
- [ ] School capacity limits (seats)
- [ ] Graduation rate from school quality (funding)
- [ ] Citizen education level affects job eligibility and income
- [ ] Education stats in info panel
- [ ] Tests pass

## Test Plan
- Unit: School at 100% funding has 90%+ graduation rate
- Unit: School at 50% funding has ~70% graduation rate
- Unit: Citizens with university degree get higher income

## Pitfalls
- Education takes years in game time; need to track student progress
- Current `education.rs` and `education_jobs.rs` exist; extend them
- University students may come from outside city (immigration)

## Relevant Code
- `crates/simulation/src/education.rs` -- education system
- `crates/simulation/src/education_jobs.rs` -- education-job linking
- `crates/simulation/src/citizen.rs` -- citizen education level
