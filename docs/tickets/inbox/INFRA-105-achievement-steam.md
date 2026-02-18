# INFRA-105: Achievement/Prestige System with Steam Integration
**Priority:** T4
**Complexity:** M (1-3 days)
**Dependencies:** none
**Source:** master_architecture.md, M5

## Description
Expand achievement system with Steam achievement integration. Current `achievements.rs` tracks milestones; add Steam API calls to unlock Steam achievements when in-game achievements trigger. Add prestige achievements (difficult, require specific strategies). Achievement notification UI. Achievement gallery showing locked/unlocked with descriptions and hints.

## Definition of Done
- [ ] Steam achievement API integration (behind feature flag)
- [ ] Achievement notification popup in-game
- [ ] Achievement gallery UI with icons
- [ ] At least 20 achievements defined
- [ ] Prestige achievements for advanced play
- [ ] Tests pass

## Test Plan
- Unit: Reaching 10K population triggers population achievement
- Unit: Steam API called when achievement unlocked (mock in tests)
- Integration: Achievement gallery shows correct lock/unlock state

## Pitfalls
- Steam API requires Steamworks SDK integration (C FFI)
- Achievements must not be trivially unlockable; require meaningful play
- Current `achievements.rs` exists; extend

## Relevant Code
- `crates/simulation/src/achievements.rs` -- achievement system
- `crates/app/src/main.rs` -- Steam API initialization
