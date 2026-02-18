# INFRA-098: Tutorial/Onboarding Flow
**Priority:** T4
**Complexity:** L (3-5 days)
**Dependencies:** none
**Source:** master_architecture.md, M5

## Description
Create guided tutorial for new players. Step-by-step introduction: 1) Camera controls, 2) Place first road, 3) Zone residential, 4) Zone commercial, 5) Place power plant, 6) Place water service, 7) Watch city grow, 8) Place school/hospital, 9) Review budget. Each step highlights the relevant UI element, locks other tools, and explains the mechanic. Skippable for experienced players.

## Definition of Done
- [ ] Tutorial sequence with 8-10 steps
- [ ] UI highlighting for relevant tools at each step
- [ ] Explanatory text/tooltips per step
- [ ] Tool locking (only relevant tool available per step)
- [ ] Skip tutorial option
- [ ] Tutorial completion flag (don't show again)
- [ ] Tests pass

## Test Plan
- Unit: Each tutorial step advances on completion of required action
- Integration: New player can complete tutorial and understand core loop

## Pitfalls
- Tutorial must work with current build (references correct UI elements)
- Locking tools may frustrate players; allow "free mode" escape
- Tutorial map should be flat and simple (no terrain challenges)

## Relevant Code
- `crates/ui/src/lib.rs` -- tutorial UI overlay
- `crates/rendering/src/input.rs` -- tool locking
