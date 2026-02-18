# CIT-034: Hedonic Adaptation (Rising Expectations)

**Priority:** T3 (Differentiation)
**Complexity:** Medium (1-2 person-weeks)
**Dependencies:** CIT-003 (income distribution)
**Source:** social_agent_simulation.md Section 5.4 (Easterlin Paradox)

## Description

Implement hedonic treadmill: as citizens get wealthier, their baseline expectations rise, requiring better conditions to maintain the same happiness level. Expectations increase with wealth tier: Poverty (none), LowerMiddle (-3), MiddleClass (-6), UpperMiddle (-10), Wealthy (-14), Elite (-18). Similar to RimWorld's expectations system. Expectations_penalty reduces raw happiness score. This ensures that wealthy neighborhoods still generate complaints and the player can't "solve" happiness permanently.

## Definition of Done

- [ ] Expectations level computed from income class
- [ ] Expectations penalty applied to raw happiness score
- [ ] Penalty values match research doc table
- [ ] Citizens with high expectations need better services to be equally happy
- [ ] Expectations visible in citizen detail panel
- [ ] City-average expectations tracked as metric

## Test Plan

- Unit test: poverty citizen with score 60 stays at 60
- Unit test: wealthy citizen with score 60 becomes 46 after expectations
- Integration test: wealthy neighborhoods show lower average happiness than expected

## Pitfalls

- Expectations penalty must not push happiness below zero
- Must balance so that wealthy citizens aren't always miserable

## Relevant Code

- `crates/simulation/src/happiness.rs` (update_happiness)
- `crates/simulation/src/wealth.rs` (WealthTier)
