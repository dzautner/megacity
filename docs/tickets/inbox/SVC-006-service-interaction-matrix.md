# SVC-006: Service Cross-Interaction Matrix

**Priority:** T2 (Depth)
**Complexity:** Medium (2 person-weeks)
**Dependencies:** SVC-001 (hybrid coverage)
**Source:** historical_demographics_services.md Section 6.4

## Description

Services should affect each other. Education (high quality) -> Police (less crime): educated population commits -15% crime. Healthcare (good coverage) -> Fire (fewer deaths). Police (good) -> Healthcare (fewer assault injuries). Education -> Healthcare (health literacy +10%). Social services -> Police (poverty reduction = -20% crime). Parks/recreation -> Healthcare (exercise +5% health). Libraries -> Education (+10% education quality). Implement as multiplier matrix applied after individual service calculations.

## Definition of Done

- [ ] `ServiceInteractionMatrix` resource with multiplier values
- [ ] Education quality affects crime rate (-15% at full education)
- [ ] Police quality affects healthcare demand (-10% trauma at full police)
- [ ] Social services affect crime (-20% at full welfare coverage)
- [ ] Parks affect health (+5% at full park coverage)
- [ ] Libraries affect education (+10% at full library coverage)
- [ ] Matrix applied in `update_service_coverage` or downstream
- [ ] Interaction effects visible in service detail panel

## Test Plan

- Unit test: full education reduces crime by 15%
- Unit test: full parks improve health by 5%
- Integration test: well-rounded service investment produces compounding returns
- Integration test: neglecting one service drags down others

## Pitfalls

- Circular interactions (A helps B helps A) can create positive feedback; use multiplicative not additive
- Matrix must be applied after base calculations to avoid order-dependence

## Relevant Code

- `crates/simulation/src/happiness.rs` (update_service_coverage)
- `crates/simulation/src/crime.rs` (update_crime)
- `crates/simulation/src/health.rs` (update_health_grid)
