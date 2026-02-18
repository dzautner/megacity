# AUDIO-018: Notification and UI Sound System

**Category:** Audio / UI
**Priority:** T4
**Source:** sound_design.md -- Section 5

## Summary

Hierarchical notification sounds with priority (Critical > Warning > Info > Progress). Earcon design: Critical (ascending dissonant, 0.8 vol), Warning (two-tone attention, 0.6 vol), Info (gentle chime, 0.4 vol), Progress (pleasant ascending, 0.5 vol). Cooldown system prevents spam. Accessibility: visual alternatives for all sounds.

## Details

- Priority hierarchy determines playback order and ducking
- Cooldown per notification type (1-5 seconds)
- Maximum concurrent notifications (3)
- Ducking: critical ducks everything, warning ducks music/ambience, info ducks nothing
- Accessibility: visual pulse, screen flash, or vibration for deaf/HoH players

## Dependencies

- AUDIO-001 (notification bus)
- Event/notification system

## Acceptance Criteria

- [ ] Priority-based notification sounds
- [ ] Cooldown prevents spam
- [ ] Ducking chain functional
- [ ] Visual alternatives available
