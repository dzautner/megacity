# AUDIO-010: Time-of-Day Musical Palettes

**Category:** Audio / Music
**Priority:** T4
**Source:** sound_design.md -- Section 3.5

## Summary

11 time-of-day periods with distinct tempo, key, and instrument palette. Pre-Dawn (solo piano, 60 BPM), Dawn (flute + piano, 68), Morning (guitar + percussion, 76), through Deep Night (drone only, 48 BPM). Transitions are gradual 15-30 minute crossfades, not hard switches.

## Details

- Periods: PreDawn, Dawn, Morning, MidMorning, Midday, Afternoon, GoldenHour, Evening, Night, LateNight, DeepNight
- Each period: target BPM, key tendency, instrument palette, mood
- BPM interpolation between periods using exponential lerp
- Key center shifts through circle of fifths
- Night music distinctly different: jazzy, ambient, minimal

## Dependencies

- AUDIO-007, AUDIO-008 (music system)
- GameClock

## Acceptance Criteria

- [ ] 11 time-of-day periods with distinct musical character
- [ ] Gradual BPM and timbral transitions
- [ ] Night music clearly different from day
- [ ] Dawn chorus moment identifiable by ear
