# AUDIO-007: Dynamic Music System -- Vertical Layering

**Category:** Audio / Music
**Priority:** T4
**Source:** sound_design.md -- Section 3.1, 3.2

## Summary

Implement stem-based adaptive music. Each music piece has 6-7 stems (Base Pad, Harmonic, Rhythmic, Bass, Melodic, Accent, Tension) that are independently mixed based on city state. All stems play simultaneously but at different volumes. Volume targets computed from population, happiness, time-of-day, activity level, crisis state.

## Details

- StemType enum: BasePad, Harmonic, Rhythmic, Bass, Melodic, Accent, Tension
- Volume targets computed per stem from CityStats, Weather, GameClock, ActiveDisaster, CityBudget
- Smooth volume tweening (2-4 bar fade durations with easing curves)
- All stems synchronized to shared BPM clock
- Tension stem suppresses Melodic stem when active
- Base Pad always present, other stems conditional

## Dependencies

- AUDIO-001 (audio bus hierarchy, Kira clock system)
- Game simulation state (CityStats, Weather, etc.)

## Acceptance Criteria

- [ ] 6-7 stems loaded and playing simultaneously
- [ ] Stem volumes respond to city state changes
- [ ] Smooth volume transitions (no pops or clicks)
- [ ] Clock-synchronized playback
