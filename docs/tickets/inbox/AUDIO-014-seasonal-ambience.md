# AUDIO-014: Seasonal Ambient Sound Beds

**Category:** Audio / Environmental
**Priority:** T4
**Source:** sound_design.md -- Section 4.2

## Summary

Four seasonal sound beds that crossfade over 2-3 game days at season transitions. Spring: dawn chorus, insect buzz, frogs. Summer: cicadas, AC hum, heat shimmer drone. Autumn: wind through dry leaves, geese, fewer insects. Winter: near-silence, wind gusts, muffled sounds, ice creaking.

## Details

- Each season has unique continuous ambient bed
- Crossfade during first 5 days of new season
- Season-specific detail sounds at appropriate times
- Spring dawn chorus peaks at 5:45 AM, bell curve volume
- Summer AC hum scales with temperature
- Winter is characterized by absence of sound

## Dependencies

- AUDIO-001 (ambience bus)
- Weather system (season)
- GameClock

## Acceptance Criteria

- [ ] Four distinct seasonal sound beds
- [ ] Smooth crossfade at season boundaries
- [ ] Season-specific detail sounds (cicadas, crickets, etc.)
- [ ] Winter silence and muffling
