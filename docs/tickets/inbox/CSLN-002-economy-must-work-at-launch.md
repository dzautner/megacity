# CSLN-002: Economy Balance Pass (No Bankruptcy Spiral)
**Priority:** T1
**Complexity:** M
**Dependencies:** ECON-001
**Source:** cities_skylines_analysis.md, section 16.7 (lesson 3)

## Description
CS2's broken economy ruined ALL other systems because players couldn't build cities large enough to engage with them. Megacity must have a balanced economy where default tax rates at any city size produce positive net income.

- Audit: at population milestones (100, 500, 2K, 10K, 50K, 100K), verify positive budget with default rates
- Ensure service costs do not exceed tax income at any milestone
- Building construction cost must have positive ROI within 5 game-years
- Starting treasury must cover at least 10 minutes of building before first tax collection
- Emergency loan available if treasury drops below $0
- Test: automated playthrough reaches 10K population without bankruptcy

## Definition of Done
- [ ] Positive budget at all population milestones with default settings
- [ ] Starting treasury sufficient for initial development
- [ ] Emergency loan prevents hard bankruptcy
- [ ] Automated economy test passes
- [ ] Economy tuning documented with expected values per milestone

## Test Plan
- Automated: Scripted playthrough to 10K pop, verify treasury > 0 at all points
- Unit: Expected income > expected expenses at 500 pop with default services

## Pitfalls
- Property tax replaces flat tax (ECON-001) -- balance must be done AFTER ECON-001
- Service costs scale differently than tax income -- crossover point must not exist
- Construction costs can drain treasury faster than income if player builds too fast

## Relevant Code
- `crates/simulation/src/economy.rs:collect_taxes` -- tax income
- `crates/simulation/src/budget.rs` -- expense tracking
- `crates/simulation/src/services.rs:ServiceBuilding::monthly_maintenance` -- service costs
