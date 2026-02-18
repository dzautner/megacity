# INFRA-122: Maintenance Debt Tracking and Infrastructure Report Card
**Priority:** T2
**Complexity:** M (1-3 days)
**Dependencies:** INFRA-079
**Source:** infrastructure_engineering.md, Section 10

## Description
Track deferred maintenance as "maintenance debt" across all infrastructure types (roads, pipes, power, transit). Every infrastructure piece has annual maintenance cost; underfunding accumulates debt. Debt accelerates deterioration (exponential, not linear). Infrastructure report card grades A-F per system. Report card affects bond ratings and loan interest rates. The 1-to-6 rule: $1 not spent on preventive maintenance costs $4-6 in reconstruction.

## Definition of Done
- [ ] Per-infrastructure-type maintenance debt accumulation
- [ ] Debt accelerates deterioration
- [ ] Infrastructure report card A-F per system
- [ ] Report card published periodically (yearly)
- [ ] Bond rating affected by report card grades
- [ ] Maintenance funding slider showing long-term consequences
- [ ] Tests pass

## Test Plan
- Unit: Underfunding road maintenance for 5 years creates proportionally higher repair costs
- Unit: Infrastructure grade drops from B to D after sustained underfunding
- Integration: Player sees report card and understands funding consequences

## Pitfalls
- Maintenance debt is invisible until crisis; need early warnings
- Different infrastructure types degrade at different rates
- Cascading failures from multiple system failures simultaneously

## Relevant Code
- `crates/simulation/src/road_maintenance.rs` -- road maintenance
- `crates/simulation/src/budget.rs` -- maintenance budget allocation
- `crates/simulation/src/loans.rs` -- bond rating interaction
