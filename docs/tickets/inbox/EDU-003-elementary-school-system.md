# EDU-003: Elementary School Enrollment and Quality

**Priority:** T2 (Depth)
**Complexity:** Low-Medium (1 person-week)
**Dependencies:** EDU-001 (education pipeline)
**Source:** historical_demographics_services.md Section 3.4

## Description

Elementary school (ages 6-10): capacity 300 students. Teacher-student ratio target 1:15 (20 teachers). Quality affected by: funding (budget allocation), teacher ratio, building condition. Graduation rate: 92% base + quality_modifier. Students without elementary school in catchment area cannot advance education. Elementary education is the minimum for most employment.

## Definition of Done

- [ ] Elementary enrollment for ages 6-10
- [ ] Capacity 300 students
- [ ] Teacher-student ratio tracking
- [ ] Quality formula: funding*0.4 + teacher_ratio*0.3 + building_condition*0.3
- [ ] Graduation rate: 92% base + quality_bonus
- [ ] Students graduate to HighSchool at age 11
- [ ] No elementary school = stuck at education level 0

## Test Plan

- Unit test: 300 students enrolled at capacity
- Unit test: quality degrades with under-funding
- Unit test: 92% base graduation rate

## Pitfalls

- Elementary schools must be distributed across city; one central school is insufficient

## Relevant Code

- `crates/simulation/src/services.rs` (ServiceType::ElementarySchool)
- `crates/simulation/src/education.rs`
