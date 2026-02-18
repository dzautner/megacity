# CIT-035: Relative Deprivation (Inequality Penalty)

**Priority:** T3 (Differentiation)
**Complexity:** Low (0.5 person-weeks)
**Dependencies:** CIT-003 (income distribution)
**Source:** social_agent_simulation.md Section 5.4

## Description

Citizens compare themselves to neighbors. If citizen income is significantly below neighborhood average, they suffer relative deprivation penalty. Formula: relative_position = citizen_income / neighborhood_avg_income. If relative_position < 0.5, penalty = (0.5 - relative_position) * 20 (max -10 happiness). This creates gentrification pressure: when wealthy citizens move into low-income area, existing residents feel relatively worse off even if their absolute conditions haven't changed.

## Definition of Done

- [ ] Neighborhood average income computed per chunk
- [ ] Relative position = citizen_income / chunk_avg_income
- [ ] Deprivation penalty for relative_position < 0.5
- [ ] Penalty capped at -10 happiness
- [ ] No penalty for citizens above neighborhood average
- [ ] Feeds into gentrification dynamics

## Test Plan

- Unit test: citizen at 50% of neighborhood average gets -10 penalty
- Unit test: citizen at 80% of neighborhood average gets no penalty
- Integration test: gentrifying neighborhood shows happiness drop for original residents

## Pitfalls

- Neighborhood average must exclude the citizen themselves to avoid self-reference
- Must handle neighborhoods with very few citizens (unstable average)

## Relevant Code

- `crates/simulation/src/happiness.rs` (update_happiness)
- `crates/simulation/src/districts.rs` (aggregate_districts)
