# AUDIO-023: Audio Mixing and Mastering Pipeline

**Category:** Audio / Technical
**Priority:** T4
**Source:** sound_design.md -- Section 9

## Summary

Implement loudness normalization (target -14 LUFS), dynamic range management, frequency band allocation, ducking chains, and master bus processing. Music: -18 LUFS, Ambience: -24 LUFS, SFX: -12 LUFS, Notifications: -10 LUFS. Ducking priority chain ensures critical sounds always audible.

## Acceptance Criteria

- [ ] Loudness normalization applied per bus
- [ ] Ducking chains functional (notification ducks music, etc.)
- [ ] Master bus limiter prevents clipping
- [ ] Frequency bands allocated to avoid masking
