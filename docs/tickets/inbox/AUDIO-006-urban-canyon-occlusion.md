# AUDIO-006: Sound Occlusion and Urban Canyon Effects

**Category:** Audio / Spatial
**Priority:** T5
**Source:** sound_design.md -- Section 2.5

## Summary

Dense building areas create urban canyons that affect sound propagation. Simple geometric analysis of building density around sound sources to apply reverb (canyon echo) and low-pass filtering (building occlusion). Not full ray-traced audio but approximation using grid data.

## Details

- Building density around emitter determines reverb amount (dense = more echo)
- Urban canyon detection: parallel rows of buildings create canyon
- Occluded sounds (behind buildings) get low-pass filter
- Open spaces (parks, water) have less reverb, more clarity
- Simplified model using chunk-level building density, not per-building geometry

## Dependencies

- AUDIO-001 (audio bus hierarchy)
- Building/Grid system

## Acceptance Criteria

- [ ] Dense areas have more reverb
- [ ] Open areas have clearer sound
- [ ] Occluded sources are filtered
