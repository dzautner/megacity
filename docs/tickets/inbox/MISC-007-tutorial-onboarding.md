# MISC-007: Tutorial/Onboarding Flow
**Priority:** T4
**Complexity:** L
**Dependencies:** none
**Source:** master_architecture.md, section M5

## Description
Create a guided tutorial that teaches new players the core loop: place roads, zone areas, provide services, manage budget. Without onboarding, city builders are impenetrable to new players.

Tutorial steps:
1. Place your first road (teach road tool)
2. Zone residential and commercial areas (teach zone painting)
3. Build a water pump and power plant (teach utilities)
4. Watch your first citizens move in
5. Build a school and fire station (teach services)
6. Check your budget (teach budget panel)
7. Handle a traffic problem (teach road hierarchy)
8. Congratulations: you've built your first city!

## Definition of Done
- [ ] 8-step tutorial flow
- [ ] Each step has highlight/arrow pointing to relevant UI
- [ ] Step advances automatically when condition met
- [ ] Tutorial skippable
- [ ] Tutorial replayable from settings

## Test Plan
- Integration: Complete tutorial from start, verify all steps work

## Pitfalls
- Tutorial must not prevent free play (skip option essential)
- Tutorial conditions must be simple to detect
- UI highlighting needs overlay system

## Relevant Code
- `crates/ui/src/lib.rs` -- tutorial overlay system
- `crates/simulation/src/events.rs` -- tutorial events
