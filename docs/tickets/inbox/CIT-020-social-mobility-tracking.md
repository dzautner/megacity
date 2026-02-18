# CIT-020: Social Mobility Tracking

**Priority:** T3 (Differentiation)
**Complexity:** Low-Medium (1-2 person-weeks)
**Dependencies:** CIT-001 (demographics), CIT-003 (income distribution)
**Source:** social_agent_simulation.md Section 6

## Description

Track income quintile transitions over time to measure social mobility. Each citizen's starting income quintile (at age 18) is recorded. Compare against current quintile to measure upward/downward mobility. City-level social mobility index = fraction of citizens who moved up at least one quintile. High education quality and job diversity should increase mobility. Gentrification and inequality should decrease it.

## Definition of Done

- [ ] Birth/starting income quintile stored per citizen
- [ ] Current income quintile computed per citizen
- [ ] Social mobility index = fraction who moved up 1+ quintiles
- [ ] Downward mobility tracked separately
- [ ] City-level mobility index displayed in stats UI
- [ ] Mobility by demographic group (education, ethnicity) tracked
- [ ] Historical mobility data over time (graphable)

## Test Plan

- Unit test: citizen moving from Q1 to Q3 counts as upward mobility
- Unit test: citizen staying in Q1 counts as stagnant
- Integration test: city with good schools has higher upward mobility

## Pitfalls

- Quintile boundaries shift as income distribution changes; use absolute thresholds or relative
- Young citizens haven't had time for mobility; exclude < 25 from index

## Relevant Code

- `crates/simulation/src/wealth.rs` (WealthStats, income tiers)
- `crates/simulation/src/citizen.rs` (CitizenDetails.salary)
