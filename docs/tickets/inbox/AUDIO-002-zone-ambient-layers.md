# AUDIO-002: Zone-Based Ambient Sound Layers

**Category:** Audio / Spatial
**Priority:** T4
**Source:** sound_design.md -- Section 2.1

## Summary

Implement zone-based ambient sound system that mixes ambient layers based on what cell types are visible to the camera. Residential zones produce domestic sounds, commercial produce bustle, industrial produce machinery, parks produce nature. Volume weighted by cell count in camera frustum.

## Details

- Sample visible chunks (8x8 cells) to determine surface composition
- Per-zone ambient loops: residential (domestic hum, HVAC), commercial (bustle, chatter, registers), industrial (machinery, forges, trucks), park (birds, rustling leaves, fountain)
- Time-of-day modulation: residential quieter at day (empty), louder at evening; commercial peak daytime
- Camera distance scaling: close zoom = individual sounds, far zoom = blended wash
- Smooth crossfading when camera pans between zones

## Dependencies

- AUDIO-001 (audio bus hierarchy)
- Grid/Zones system

## Acceptance Criteria

- [ ] Zone ambient layers active and mixed by camera view
- [ ] Time-of-day modulation functional
- [ ] Smooth crossfading when panning
- [ ] Camera distance affects mix (close vs far)
