# CIT-054: Labor Market with Supply/Demand Wages

**Priority:** T3 (Differentiation)
**Complexity:** Medium (3-4 person-weeks)
**Dependencies:** CIT-002 (education levels)
**Source:** master_architecture.md Section 1.8

## Description

Labor market with wage determination based on supply and demand per education/skill tier. When demand > supply (more jobs than workers at a skill level), wages rise. When supply > demand, wages fall. Wage stickiness: wages rise faster than they fall (behavioral economics). Minimum wage policy as a floor. Labor market tightness metric per skill tier. Tight labor market attracts immigration, raises wages, reduces business profitability. Loose market reduces wages, increases inequality.

## Definition of Done

- [ ] Per-skill-tier labor supply and demand tracking
- [ ] Wage adjustment based on supply/demand ratio
- [ ] Wage stickiness (asymmetric adjustment speed)
- [ ] Minimum wage policy
- [ ] Labor market tightness metric per tier
- [ ] Wages affect: citizen income, business costs, zone profitability
- [ ] Immigration response to tight labor market
- [ ] Labor statistics in economy panel

## Test Plan

- Unit test: more demand than supply raises wages
- Unit test: minimum wage prevents wages falling below floor
- Unit test: wage stickiness (slower to fall than rise)
- Integration test: office zone construction without university grads = rising office wages

## Pitfalls

- Wage adjustments should be gradual (monthly, not per-tick)
- Must not create wage spiral (wages -> costs -> prices -> wages)

## Relevant Code

- `crates/simulation/src/economy.rs`
- `crates/simulation/src/education_jobs.rs`
- `crates/simulation/src/citizen.rs` (salary)
