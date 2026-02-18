# INFRA-142: Roguelite Meta-Progression Mode
**Priority:** T5
**Complexity:** M (1-3 days)
**Dependencies:** none
**Source:** master_architecture.md, M6

## Description
Add roguelite meta-progression mode: each playthrough earns meta-currency based on achievements. Meta-currency unlocks permanent bonuses for future playthroughs (starting budget, technology unlocks, building discounts). Each playthrough uses random seed with random challenges. Failed cities still earn some meta-currency. Creates replayability loop.

## Definition of Done
- [ ] Meta-currency earned from playthrough achievements
- [ ] Meta-progression unlock tree
- [ ] Permanent bonuses applied at game start
- [ ] Random challenges per playthrough
- [ ] Meta-progression save separate from city save
- [ ] Tests pass

## Test Plan
- Unit: Completing achievement earns meta-currency
- Unit: Unlocked bonus applies at next game start
- Integration: Meta-progression makes subsequent playthroughs distinct

## Pitfalls
- Permanent bonuses must not trivialize the game
- Must balance between rewarding progress and maintaining challenge
- Meta-save corruption should not affect city saves

## Relevant Code
- `crates/simulation/src/achievements.rs` -- achievement tracking
- `crates/save/src/lib.rs` -- meta-save format
