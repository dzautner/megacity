# AUDIO-012: Musical Stinger System

**Category:** Audio / Music
**Priority:** T4
**Source:** sound_design.md -- Section 3.7

## Summary

Short (2-8 second) musical phrases punctuating events: population milestone (triumphant fanfare), building complete (gentle chime), disaster starts (timpani hit), achievement (ascending arpeggio), budget surplus (ascending strings), budget deficit (descending horn). Queue-based with cooldowns and priority, max 3 queued, auto-duck music by 3-6 dB.

## Details

- StingerSystem with queue, cooldown tracking, priority sorting
- Stinger types: PopulationMilestone, BuildingComplete, DisasterStart, DisasterEnd, PolicyEnacted, Achievement, FirstZoneType, BudgetSurplus, BudgetDeficit, Festival, NewConnection
- Priority levels: Low (0.7 vol), Medium (0.85), High (1.0)
- Cooldown per stinger type to prevent spam
- Music stems ducked during stinger playback

## Dependencies

- AUDIO-007 (music system for ducking)
- Event system

## Acceptance Criteria

- [ ] Stingers play on appropriate events
- [ ] Cooldown prevents spam
- [ ] Priority queue functional
- [ ] Music ducking during stinger
