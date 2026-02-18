# EDU-005: University Enrollment and Research

**Priority:** T2 (Depth)
**Complexity:** Medium (1-2 person-weeks)
**Dependencies:** EDU-001 (education pipeline)
**Source:** historical_demographics_services.md Section 3.4

## Description

University (ages 18-22+): capacity 5000 students. Provides Bachelors (4 years), Masters (+2 years), Doctorate (+3 years). Enrollment rate depends on family income and city education culture (see CIT-008). University provides research bonus: +10% tech advancement speed, +5% innovation rate. University graduates demand office/professional zone jobs. University attracts immigration from outside (education-motivated migration). University prestige metric based on funding and faculty quality.

## Definition of Done

- [ ] University enrollment for ages 18+
- [ ] Capacity 5000 students
- [ ] 4-year bachelor program, 2-year masters, 3-year doctorate
- [ ] Enrollment rate by income class
- [ ] Graduation rate 75% bachelors, 85% masters, 70% doctorate
- [ ] Research bonus to tech advancement
- [ ] University graduates demand appropriate jobs
- [ ] University prestige metric

## Test Plan

- Unit test: enrollment rate varies by income
- Unit test: 4-year program produces bachelors graduates
- Unit test: research bonus applied with university

## Pitfalls

- University students are not working; large university = smaller workforce temporarily
- Multiple education levels (B/M/D) in one building adds complexity

## Relevant Code

- `crates/simulation/src/services.rs` (ServiceType::University)
- `crates/simulation/src/education_jobs.rs`
