# EDU-004: High School Enrollment and Dropout Rates

**Priority:** T2 (Depth)
**Complexity:** Low-Medium (1 person-week)
**Dependencies:** EDU-001 (education pipeline)
**Source:** historical_demographics_services.md Section 3.4

## Description

High school (ages 11-17): capacity 1500 students. Teacher-student ratio target 1:20. Higher dropout risk than elementary (2-8% annual, see CIT-007). Quality affected by: funding, teacher ratio, extracurricular programs. Graduation enables office/professional jobs. High school sports fields and stadiums provide entertainment bonus. Lab/library facilities improve quality.

## Definition of Done

- [ ] High school enrollment for ages 11-17
- [ ] Capacity 1500 students
- [ ] Dropout rate 2-8% annual (modified by factors from CIT-007)
- [ ] Quality formula including extracurricular facilities
- [ ] Graduation enables HighSchool education level
- [ ] SportsField adjacent to school provides bonus
- [ ] Library adjacent to school provides bonus

## Test Plan

- Unit test: capacity of 1500
- Unit test: dropout rate in 2-8% range
- Unit test: adjacent sports field improves quality

## Pitfalls

- Dropout rate must be meaningful but not devastating to city workforce

## Relevant Code

- `crates/simulation/src/services.rs` (ServiceType::HighSchool)
- `crates/simulation/src/education.rs`
