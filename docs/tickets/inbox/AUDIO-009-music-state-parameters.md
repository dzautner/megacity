# AUDIO-009: Music City State Parameter Sampling

**Category:** Audio / Music
**Priority:** T4
**Source:** sound_design.md -- Section 3.4

## Summary

Implement MusicStateSnapshot that samples simulation state once per beat. Drives music system with: PopulationTier (Village/Town/City/Metropolis/Megacity), CityMood (Struggling/Neutral/Content/Thriving/Euphoric), TimeOfDayPeriod (11 periods), Season, CrisisLevel (None/Minor/Major/Catastrophic), ActivityLevel.

## Details

- PopulationTier determines arrangement density and instrument count
- CityMood determines key/mode selection (major vs minor)
- TimeOfDayPeriod drives BPM and instrument palette
- CrisisLevel overrides with tension/urgency
- Sample once per beat, not per frame, to prevent jitter

## Dependencies

- AUDIO-007 (vertical layering)
- CityStats, Weather, GameClock, ActiveDisaster, CityBudget

## Acceptance Criteria

- [ ] MusicStateSnapshot computed per beat
- [ ] Population tier affects arrangement density
- [ ] City mood affects harmonic content
- [ ] Time-of-day changes instrument palette and tempo
