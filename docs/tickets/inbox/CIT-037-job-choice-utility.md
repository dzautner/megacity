# CIT-037: Job Choice Utility Model

**Priority:** T3 (Differentiation)
**Complexity:** Medium (2 person-weeks)
**Dependencies:** CIT-002 (education levels)
**Source:** social_agent_simulation.md Section 4.3

## Description

Job choice utility model for unemployed and dissatisfied workers. Job utility = salary_weight * salary_score + commute_weight * commute_inverse + match_weight * education_match + prestige_weight * zone_prestige. Workers accept jobs when utility > threshold. Workers change jobs when alternative job utility exceeds current by >15%. Job search radius increases with unemployment duration. Over-qualified workers accept lower-prestige jobs after 6+ months unemployed.

## Definition of Done

- [ ] `job_utility()` function with weighted factors
- [ ] Salary score relative to education-appropriate wage
- [ ] Commute distance penalty
- [ ] Education match bonus/penalty
- [ ] Job search with increasing radius over time
- [ ] Voluntary job change when better option available
- [ ] Over-qualification acceptance after prolonged unemployment

## Test Plan

- Unit test: high salary job outweighs longer commute
- Unit test: education match provides utility bonus
- Unit test: long-unemployed citizen accepts lower jobs
- Integration test: workers commute to higher-paying distant jobs

## Pitfalls

- Job change churning can destabilize employment stats; add cooldown
- Must coordinate with existing job_matching in education_jobs.rs

## Relevant Code

- `crates/simulation/src/education_jobs.rs` (job_matching, assign_workplace_details)
- `crates/simulation/src/citizen.rs` (WorkLocation)
