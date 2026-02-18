# CIT-010: Life Stage -- Senior (55-64) Pre-Retirement

**Priority:** T2 (Depth)
**Complexity:** Low (0.5 person-weeks)
**Dependencies:** None
**Source:** social_agent_simulation.md Section 2.2

## Description

Senior (55-64) is the pre-retirement stage. Early retirement probability 5-15% annually based on savings sufficiency and health. Reduced mobility (5% annual relocation). Higher healthcare utilization (3x young adult). Widowhood begins for partnered seniors. Working but slowing career (no further promotions typical).

## Definition of Done

- [ ] Early retirement probability check each game-year
- [ ] Retirement triggered by sufficient savings OR poor health
- [ ] Healthcare utilization multiplier (3x) for demand calculation
- [ ] Widowhood probability when partner dies
- [ ] Reduced relocation probability (5%)
- [ ] Transition to Retired at age 65 (90%+ probability)

## Test Plan

- Unit test: early retirement probability in 5-15% range
- Unit test: poor health increases early retirement probability
- Integration test: seniors with high savings retire earlier

## Pitfalls

- Forced retirement at 65 may conflict with ongoing work assignments
- Widowhood must properly dissolve household/family relationships

## Relevant Code

- `crates/simulation/src/citizen.rs` (LifeStage::Senior, line 22)
- `crates/simulation/src/life_simulation.rs` (retire_workers)
