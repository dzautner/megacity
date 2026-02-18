# AUDIO-017: Disaster Sound Design

**Category:** Audio / Environmental
**Priority:** T4
**Source:** sound_design.md -- Section 4.5

## Summary

Multi-phase disaster sound design. Tornado: warning rumble -> roaring wind + debris -> aftermath silence. Earthquake: sub-bass rumble + structure groaning -> cracking/crumbling -> aftershock tremors. Flood: rising water rush + emergency sirens. Fire: crackling + sirens. Each disaster has warning, active, and aftermath phases.

## Details

- Tornado: spatial to tornado position, 100-500 Hz dominant, debris per destroyed cell
- Earthquake: 30-80 Hz sub-bass, rapid tremolo on all sounds, 3-phase
- Flood: rising pitch water rush, lapping against buildings
- Fire: crackling + structure groaning, fire truck sirens
- All disasters override normal ambient, reduce to eerie quiet in aftermath

## Dependencies

- AUDIO-001
- ActiveDisaster system

## Acceptance Criteria

- [ ] Each disaster type has distinct sound design
- [ ] Warning phase precedes damage phase
- [ ] Aftermath has eerie quiet period
- [ ] Spatial positioning for localized disasters
