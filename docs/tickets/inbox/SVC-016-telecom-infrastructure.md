# SVC-016: Telecom Infrastructure (Cell Towers, Data Centers)

**Priority:** T2 (Depth)
**Complexity:** Low (0.5 person-weeks)
**Dependencies:** None
**Source:** master_architecture.md Section 1.10

## Description

Telecom infrastructure provides modern city services. CellTower: wireless coverage radius, enables work-from-home for office workers, reduces commute demand by 5-10%. DataCenter: enables tech industry, attracts office zone demand, large power consumer. Telecom coverage is a modern necessity: lack of coverage reduces happiness (-3) and office productivity (-10%). Coverage already tracked via COVERAGE_TELECOM bitflag.

## Definition of Done

- [ ] CellTower: coverage radius, work-from-home enablement
- [ ] DataCenter: tech industry attraction, office demand boost
- [ ] Work-from-home probability for office workers in covered areas
- [ ] Office productivity penalty without telecom coverage
- [ ] Telecom coverage in happiness calculation (replace flat TELECOM_BONUS)
- [ ] Power consumption for DataCenter (large)

## Test Plan

- Unit test: cell tower enables work-from-home for covered area
- Unit test: data center boosts office demand
- Unit test: missing telecom penalizes office productivity

## Pitfalls

- Work-from-home reduces commute traffic; good side effect to model
- DataCenter power consumption should be significant

## Relevant Code

- `crates/simulation/src/services.rs` (CellTower, DataCenter)
- `crates/simulation/src/happiness.rs` (TELECOM_BONUS)
