# EDU-007: Education-Workforce Link (Over/Under Education)

**Priority:** T2 (Depth)
**Complexity:** Medium (1-2 person-weeks)
**Dependencies:** CIT-002 (education levels), EDU-001 (education pipeline)
**Source:** social_agent_simulation.md Section 5, master_architecture.md

## Description

Job matching should account for education requirements per zone type. Industrial requires elementary-high school. Commercial requires high school-bachelors. Office requires bachelors-masters. Over-educated workers (PhD in industrial job) suffer happiness penalty (-5) and productivity penalty. Under-educated workers (elementary in office) have reduced productivity (-30%) and no promotion. Education mismatch creates pressure for zone mix changes.

## Definition of Done

- [ ] Job education requirements per zone/occupation type
- [ ] Over-education happiness penalty (-5)
- [ ] Under-education productivity penalty (-30%)
- [ ] Education-job match quality metric per citizen
- [ ] City-wide education mismatch stat
- [ ] Zone demand adjusted by workforce education (office demand rises with educated workforce)
- [ ] Warning when education mismatch is high

## Test Plan

- Unit test: PhD in factory = over-educated penalty
- Unit test: elementary education in office = under-educated penalty
- Integration test: university city generates office zone demand

## Pitfalls

- Over-education penalty may discourage player from building universities; balance carefully
- Must handle transition period when city upgrades from industrial to office economy

## Relevant Code

- `crates/simulation/src/education_jobs.rs` (job_matching)
- `crates/simulation/src/zones.rs` (zone demand)
