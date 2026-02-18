# EDU-001: Education Pipeline (K-12 + University)

**Priority:** T2 (Depth)
**Complexity:** Medium (2-3 person-weeks)
**Dependencies:** CIT-002 (education levels)
**Source:** historical_demographics_services.md Section 3.4, master_architecture.md Section 1.10

## Description

Full education pipeline: Kindergarten (age 3-5, capacity 100/building) -> ElementarySchool (6-10, capacity 300) -> HighSchool (11-17, capacity 1500) -> University (18-22+, capacity 5000). Each level has quality metric (0-100) from: funding, teacher-student ratio, building condition. Graduation rate depends on quality: base 85% + quality_factor*10%. Students assigned to nearest school with available capacity. Over-capacity schools have degraded quality. Education quality feeds into land value (+10 per quality point) and workforce quality.

## Definition of Done

- [ ] Student enrollment tracking per school building
- [ ] Age-appropriate school assignment
- [ ] School capacity limits (100/300/1500/5000 by type)
- [ ] Education quality metric per school
- [ ] Graduation rate formula (base 85% + quality*10%)
- [ ] Over-capacity quality degradation
- [ ] Teacher-student ratio affecting quality (target: 1:15 elem, 1:20 high, 1:30 uni)
- [ ] Education quality -> land value bonus
- [ ] School district visualization

## Test Plan

- Unit test: age 7 assigned to elementary school
- Unit test: full school degrades quality
- Unit test: graduation rate matches formula
- Integration test: building schools improves district education level over time

## Pitfalls

- School assignment needs to respect road network (not Euclidean distance)
- Over-enrolled schools shouldn't crash; gracefully degrade

## Relevant Code

- `crates/simulation/src/education.rs` (EducationGrid, propagate_education)
- `crates/simulation/src/education_jobs.rs`
- `crates/simulation/src/services.rs` (Kindergarten, ElementarySchool, HighSchool, University)
