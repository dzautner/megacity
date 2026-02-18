# AUDIO-015: Water Body Audio

**Category:** Audio / Environmental
**Priority:** T4
**Source:** sound_design.md -- Section 4.3

## Summary

Water cells produce ambient sounds varying by type: river (rushing/babbling), lake (gentle lapping), ocean (waves crashing, gulls), fountain (splashing), frozen (ice creaking in winter). Context detected from water cell patterns (narrow line = river, large contiguous = lake, map edge = ocean).

## Dependencies

- AUDIO-001
- Grid (water cell detection)
- Weather (frozen state)

## Acceptance Criteria

- [ ] Water sounds play near water cells
- [ ] Sound character matches water type
- [ ] Frozen water in winter produces ice sounds
- [ ] Weather affects water sound (storms = louder waves)
