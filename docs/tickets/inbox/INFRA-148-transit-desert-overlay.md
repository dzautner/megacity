# INFRA-148: Transit Desert Detection and Overlay
**Priority:** T2
**Complexity:** S (hours)
**Dependencies:** INFRA-039
**Source:** infrastructure_engineering.md, Section 2 (Transit Deserts)

## Description
Identify transit deserts: areas with no transit service within 400-800m walking distance. Low-income residents disproportionately affected. Display transit desert overlay showing underserved areas. Happiness and economic penalties in transit deserts. Advisor recommendations to extend service to underserved areas. Equity metric: percentage of low-income households in transit deserts.

## Definition of Done
- [ ] Transit desert detection (no stop within 800m)
- [ ] Transit desert overlay mode
- [ ] Happiness penalty for transit desert residents
- [ ] Equity metric in stats (low-income in transit deserts)
- [ ] Advisor recommendation to serve transit deserts
- [ ] Tests pass

## Test Plan
- Unit: Area 1km from any stop is flagged as transit desert
- Unit: Happiness penalty applied to desert residents
- Integration: Advisor suggests extending bus route to transit desert

## Pitfalls
- Walking distance is network distance, not Euclidean
- Low-income identification requires income class (INFRA-140)
- Transit desert in industrial zone is less impactful than in residential

## Relevant Code
- `crates/simulation/src/services.rs` -- coverage analysis
- `crates/simulation/src/advisors.rs` -- advisor recommendations
