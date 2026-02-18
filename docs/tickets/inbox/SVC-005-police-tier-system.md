# SVC-005: Police Service Multi-Tier System

**Priority:** T2 (Depth)
**Complexity:** Medium (1-2 person-weeks)
**Dependencies:** SVC-002 (capacity limits), CRIME-001 (crime types)
**Source:** historical_demographics_services.md Section 3.2

## Description

Differentiate police service tiers. PoliceKiosk: 10 officers, foot patrol, 1-mile radius, presence deters property crime. PoliceStation: 30-50 officers, patrol cars, 3-mile radius, responds to all crime types. PoliceHQ: 100+ officers, detective unit, 5-mile radius, investigates organized/white-collar crime. Staffing ratio target: 2.4 officers per 1000 population (US average). Patrol coverage reduces crime in proportion to officer density. Detective capacity needed for case resolution.

## Definition of Done

- [ ] PoliceKiosk: 10 officers, foot patrol radius
- [ ] PoliceStation: 30-50 officers, vehicle patrol
- [ ] PoliceHQ: 100+ officers, detective capability
- [ ] Staffing ratio metric (officers per 1000 pop)
- [ ] Under-staffing reduces effectiveness proportionally
- [ ] Patrol density = officers * budget_factor / area
- [ ] Detective capacity for case investigation (CRIME-004 integration)
- [ ] Crime reduction proportional to patrol density

## Test Plan

- Unit test: PoliceKiosk deters property crime but not organized crime
- Unit test: PoliceHQ can investigate white-collar crime
- Unit test: under-staffed station has reduced crime reduction
- Integration test: staffing ratio affects city-wide crime rate

## Pitfalls

- Police buildings exist but may not all have distinct gameplay effects currently
- Officer count interacts with employment system

## Relevant Code

- `crates/simulation/src/crime.rs` (police reduction logic, lines 62-91)
- `crates/simulation/src/services.rs` (PoliceKiosk, PoliceStation, PoliceHQ)
