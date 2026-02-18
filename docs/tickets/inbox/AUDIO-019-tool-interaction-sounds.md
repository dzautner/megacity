# AUDIO-019: Tool and Interaction Sounds

**Category:** Audio / UX
**Priority:** T4
**Source:** sound_design.md -- Section 6

## Summary

Satisfying interaction sounds for all player tools. Road placement: click-drag with snap sounds. Zoning: brush sound with zone-colored pitch (residential=warm, commercial=bright, industrial=heavy). Bulldoze: crunch/crash + debris. Building placement: thud + construction start. Menu: subtle clicks and hovers. Camera: subtle whoosh on fast pan/zoom.

## Details

- Road placement: initial click, continuous drag (pitch rises with road length), snap tone at intersection, completion chime
- Zoning: brush start, continuous painting sound, zone-specific pitch/timbre, undo sound
- Bulldoze: impact sound scaled by building size, debris settling, "are you sure?" confirmation for large demolitions
- Building placement: stamp/thud with building-size scaling, construction sound begins
- Menu navigation: hover (soft tick), click (satisfying pop), open/close (swoosh)

## Dependencies

- AUDIO-001 (SFX bus)
- Input/tool system

## Acceptance Criteria

- [ ] Each tool has distinct, satisfying sounds
- [ ] Zone painting has zone-specific pitch
- [ ] Bulldoze intensity scales with building size
- [ ] All interactions have audio feedback
