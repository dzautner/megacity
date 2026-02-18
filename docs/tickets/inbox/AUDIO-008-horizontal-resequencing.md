# AUDIO-008: Dynamic Music System -- Horizontal Re-Sequencing

**Category:** Audio / Music
**Priority:** T4
**Source:** sound_design.md -- Section 3.3

## Summary

Implement section-based music sequencing. Each piece divided into sections (Intro, MainA, DevelopmentB, ContrastC, Bridge, Climax, Reflective, Tension). MusicSequencer evaluates city mood, time-of-day, and disaster state to determine section transitions. Transitions quantized to bar boundaries with crossfade.

## Details

- Section types with different durations and characters
- Transition rules: disaster overrides everything, night -> reflective, normal cycling A->B->C->Bridge->A
- Minimum section length before transition allowed
- Crossfade at bar boundaries (4-beat quantization)
- One full bar crossfade duration with easing

## Dependencies

- AUDIO-007 (vertical layering, Kira clock)

## Acceptance Criteria

- [ ] Music sections transition based on city state
- [ ] Transitions quantized to bar boundaries
- [ ] Smooth crossfades between sections
- [ ] Disaster triggers immediate section change
