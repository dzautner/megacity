# INFRA-091: Sound System (Spatial Audio, Dynamic Music, UI Sounds)
**Priority:** T4
**Complexity:** XL (1-2 weeks)
**Dependencies:** none
**Source:** master_architecture.md, M5

## Description
Implement full sound system. Spatial audio: traffic sounds from roads, construction sounds from building sites, sirens from emergency vehicles. Dynamic music: ambient soundtrack that shifts with game state (calm during building, tense during disasters, triumphant at milestones). UI sounds: button clicks, zone painting, road placement, notifications. Bevy audio plugin integration.

## Definition of Done
- [ ] Spatial audio for in-world sound sources
- [ ] Dynamic music system responsive to game state
- [ ] UI sounds for all major interactions
- [ ] Volume controls (master, music, SFX, UI)
- [ ] Mute option
- [ ] Tests pass

## Test Plan
- Unit: Traffic sound plays at road location, volume decreases with camera distance
- Integration: Music changes when disaster begins

## Pitfalls
- Audio assets need to be sourced/created (licensing considerations)
- Spatial audio with many sources needs culling (only play nearby sounds)
- Dynamic music crossfading needs smooth transitions

## Relevant Code
- `crates/app/src/main.rs` -- audio plugin registration
- New crate or module for audio system
