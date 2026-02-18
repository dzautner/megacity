# AUDIO-001: bevy_kira_audio Integration and Audio Bus Hierarchy

**Category:** Audio / Foundation
**Priority:** T4
**Source:** sound_design.md -- Section 1.1, 1.2

## Summary

Integrate `bevy_kira_audio` (Kira 3.x+) as audio backend replacing default bevy_audio. Implement audio bus hierarchy: Master -> Music, Ambience, SFX, UI, Notification buses. Each bus has independent volume, optional effects, and player-configurable settings.

## Details

- Master Bus with child buses: Music, Ambience, SFX, UI, Notification
- Music Bus sub-tracks: Base Layer, Harmonic, Rhythmic, Melodic, Atmospheric Pad, Stinger
- Ambience Bus sub-tracks: Zone Ambience (Res/Com/Ind/Park/Water), Traffic, Construction, Environment
- SFX Bus: Tool sounds, building placement, demolition, notifications
- UI Bus: Menu clicks, hover, transitions
- Player volume sliders per bus
- Performance budget: 32-48 simultaneous voices

## Dependencies

- None (foundation system)

## Acceptance Criteria

- [ ] bevy_kira_audio integrated as AudioPlugin
- [ ] Bus hierarchy functional with independent volume control
- [ ] Player settings menu with per-bus volume sliders
- [ ] Performance within budget (< 5% CPU)
