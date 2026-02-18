# AUDIO-028: Audio State Save/Load Handling

**Category:** Audio / Technical
**Priority:** T4
**Source:** sound_design.md -- Section 8.8

## Summary

Reconstruct audio state after save/load. On load: determine current weather, time-of-day, season, construction state. Start appropriate ambient layers, music section, and zone sounds. Crossfade from silence to appropriate state over 2-3 seconds.

## Acceptance Criteria

- [ ] Audio state reconstructed after load
- [ ] No jarring sounds on load
- [ ] Smooth fade-in from silence
- [ ] Correct music/ambient for loaded state
