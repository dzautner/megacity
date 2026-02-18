# MISC-002: Basic Sound Effects System
**Priority:** T2
**Complexity:** L
**Dependencies:** none
**Source:** master_architecture.md, section M3, T4

## Description
Add basic sound effects for core game actions. Sound is one of the most impactful polish features and CS2 lesson #1 is that polish matters.

Minimum sounds:
- Road placement click/drag
- Zone painting brush
- Building construction start
- Building construction complete
- Service building placement
- Notification bell (milestone, disaster, budget)
- Demolition
- Ambient city noise (scales with population)
- UI button clicks

## Definition of Done
- [ ] Sound effects for 10+ game actions
- [ ] Ambient city noise that scales with population
- [ ] Volume controls in settings
- [ ] Sounds play at correct timing

## Test Plan
- Integration: Place road, hear placement sound
- Integration: Mute volume, verify no sound

## Pitfalls
- Bevy's audio system has evolved -- verify current API
- Spatial audio (sounds from specific buildings) is T4+ feature
- Sound files need to be small (no 100MB audio packs)

## Relevant Code
- `crates/app/src/main.rs` -- audio plugin setup
- `crates/rendering/src/input.rs` -- trigger sounds on actions
- New crate possible: `crates/audio/src/lib.rs`
