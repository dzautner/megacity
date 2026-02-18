# SERV-010: Daycare and Eldercare Services
**Priority:** T3
**Complexity:** M
**Dependencies:** SERV-001
**Source:** cities_skylines_analysis.md, section 14.9

## Description
Add daycare and eldercare as service building types. Daycare enables parents to work (increases workforce participation). Eldercare keeps seniors healthy longer (reduces deathcare load).

- Daycare: service building, capacity 50-200 children, coverage radius 20 cells
- Effect: parents within radius can work (increases available workforce by 5-10%)
- Eldercare: service building, capacity 50-100 seniors, coverage radius 15 cells
- Effect: seniors within radius live longer, reduced healthcare demand
- Both cost maintenance budget
- Both increase happiness for families/elderly

## Definition of Done
- [ ] Daycare and eldercare service buildings placeable
- [ ] Daycare increases workforce participation
- [ ] Eldercare reduces elderly health issues
- [ ] Both tracked in service coverage and budget

## Test Plan
- Integration: Place daycare, verify workforce percentage increases
- Integration: Place eldercare, verify elderly life expectancy increases

## Pitfalls
- lifecycle.rs handles aging/death -- eldercare must integrate
- Workforce calculation must account for daycare coverage
- citizen_spawner.rs employment logic needs daycare consideration

## Relevant Code
- `crates/simulation/src/services.rs:ServiceType` -- add Daycare, Eldercare
- `crates/simulation/src/lifecycle.rs` -- eldercare life extension
- `crates/simulation/src/education_jobs.rs` -- daycare workforce effect
