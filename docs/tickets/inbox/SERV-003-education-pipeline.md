# SERV-003: Education Pipeline with Graduation Rates
**Priority:** T2
**Complexity:** M
**Dependencies:** SERV-001
**Source:** cities_skylines_analysis.md, section 9; master_architecture.md, section M3

## Description
Implement multi-stage education pipeline: Elementary -> High School -> University. Citizens progress through stages with graduation rates. Education level affects job eligibility, income, and citizen behavior.

- Elementary: ages 6-11, 6 year program, graduation rate 95%
- High school: ages 12-17, 6 year program, graduation rate 85%
- University: ages 18-22, 4 year program, graduation rate 70%
- Graduation rate affected by: school quality (funding level), class size (capacity utilization), happiness
- Education level stored on citizen: None, Elementary, HighSchool, University
- Job matching: Industrial accepts all, Office prefers University, Commercial prefers HighSchool+
- Over-educated workers: University graduates in Industrial jobs = unhappy

## Definition of Done
- [ ] Citizens progress through education stages
- [ ] Graduation rates computed from school quality
- [ ] Education level affects job eligibility
- [ ] Over-educated workers have happiness penalty
- [ ] Education statistics in city panel

## Test Plan
- Unit: Well-funded school has graduation rate > 80%
- Unit: Overcrowded school has lower graduation rate
- Integration: Build university, verify educated workforce percentage increases over time

## Pitfalls
- Education takes real-time years (game-time compression needed)
- Must handle schools filling up (wait list, children go uneducated)
- education.rs already exists with basic implementation -- extend, don't rewrite

## Relevant Code
- `crates/simulation/src/education.rs` -- education level tracking
- `crates/simulation/src/education_jobs.rs` -- job matching by education
- `crates/simulation/src/citizen.rs:Citizen` -- education_level field
