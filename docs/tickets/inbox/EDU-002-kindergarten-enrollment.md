# EDU-002: Kindergarten Enrollment System

**Priority:** T2 (Depth)
**Complexity:** Low (0.5 person-weeks)
**Dependencies:** EDU-001 (education pipeline)
**Source:** historical_demographics_services.md Section 3.4

## Description

Children aged 3-5 enroll in kindergarten if available within catchment radius. Kindergarten capacity: 100 children per facility. Kindergarten attendance improves school readiness (+5% graduation rate in elementary). Lack of kindergarten forces parent to stay home (reduces household income). Kindergarten is optional but beneficial service.

## Definition of Done

- [ ] Children aged 3-5 assigned to nearest kindergarten
- [ ] Kindergarten capacity 100
- [ ] Attendance bonus: +5% elementary graduation rate
- [ ] Missing kindergarten: parent stays home (income penalty)
- [ ] Kindergarten enrollment stat
- [ ] Childcare demand metric

## Test Plan

- Unit test: child age 4 enrolled in nearest kindergarten
- Unit test: full kindergarten turns away new enrollees
- Unit test: enrolled children have better elementary outcomes

## Pitfalls

- "Parent stays home" mechanic requires tracking which parent is the caregiver

## Relevant Code

- `crates/simulation/src/services.rs` (ServiceType::Kindergarten)
- CIT-006 (child life stage)
