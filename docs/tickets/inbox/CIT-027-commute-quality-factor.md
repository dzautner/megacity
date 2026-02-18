# CIT-027: Happiness Factor -- Commute Quality

**Priority:** T2 (Depth)
**Complexity:** Low (0.5 person-weeks)
**Dependencies:** None
**Source:** social_agent_simulation.md Section 5.2

## Description

Commute quality happiness component based on actual travel time. Currently uses SHORT_COMMUTE_BONUS (flat +10 if has work). Replace with: commute_happiness = 1.0 - (commute_minutes / 60.0).clamp(0.0, 1.0). <15 min = 0.75+, 15-30 min = 0.50-0.75, 30-60 min = 0.25-0.50, >60 min = 0.0-0.25. Track actual commute time from path length and traffic speed. Weight in overall happiness: 0.15.

## Definition of Done

- [ ] `CommuteDuration` component tracking actual commute time
- [ ] Commute time computed from path length / average road speed
- [ ] Traffic delay factored into commute time
- [ ] Commute quality formula: 1.0 - (minutes / 60).clamp(0, 1)
- [ ] Weight of 0.15 in overall happiness
- [ ] Replace flat SHORT_COMMUTE_BONUS
- [ ] Citizens without jobs get neutral commute score (0.5)

## Test Plan

- Unit test: 10-minute commute produces quality ~0.83
- Unit test: 45-minute commute produces quality ~0.25
- Unit test: unemployed citizen gets 0.5 commute quality
- Integration test: citizens in traffic-heavy areas report lower commute quality

## Pitfalls

- Commute duration changes each trip; use rolling average
- Citizens who walk to work should have short commutes even without roads

## Relevant Code

- `crates/simulation/src/happiness.rs` (SHORT_COMMUTE_BONUS, line 156)
- `crates/simulation/src/movement.rs` (path following)
