# AUDIO-024: Wind Simulation Audio

**Category:** Audio / Environmental
**Priority:** T4
**Source:** sound_design.md -- Section 4.6

## Summary

Wind audio driven by WindState resource. Wind direction affects stereo panning. Wind speed affects volume and character. Urban canyon amplification (1.3x in canyons, 0.8x in dense blocked areas). Wind gusts as random transient events layered on continuous wind bed.

## Dependencies

- AUDIO-001
- Wind system (WindState)
- Building grid (canyon detection)

## Acceptance Criteria

- [ ] Wind volume proportional to wind speed
- [ ] Wind direction creates stereo panning
- [ ] Urban canyon amplification functional
- [ ] Random gust events
