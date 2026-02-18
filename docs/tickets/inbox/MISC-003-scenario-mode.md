# MISC-003: Scenario/Challenge Mode
**Priority:** T4
**Complexity:** L
**Dependencies:** SAVE-001
**Source:** master_architecture.md, section M5

## Description
Create pre-built city scenarios with specific challenges. Players load a scenario and must achieve goals within constraints. Provides structured gameplay beyond sandbox.

Scenarios (5 minimum):
1. Traffic Crisis: city with terrible traffic, must fix without demolishing
2. Budget Bailout: bankrupt city, must reach positive budget in 2 game-years
3. Growth Challenge: reach 50K population from 5K in 5 game-years
4. Green City: reduce pollution to zero while maintaining economy
5. Disaster Recovery: city damaged by earthquake, rebuild to pre-disaster state

## Definition of Done
- [ ] 5 pre-built scenario save files
- [ ] Goal tracking per scenario
- [ ] Win/loss conditions
- [ ] Scenario selection menu
- [ ] Completion rewards (achievements)

## Test Plan
- Integration: Load scenario, achieve goal, verify win condition triggers

## Pitfalls
- Scenarios are curated save files -- need save versioning (SAVE-001)
- Goals need clear UI indicators (progress bar, countdown)
- Must not make scenarios too hard or too easy

## Relevant Code
- `crates/save/src/lib.rs` -- scenario save loading
- `crates/simulation/src/achievements.rs` -- goal tracking
- `crates/ui/src/toolbar.rs` -- scenario menu
