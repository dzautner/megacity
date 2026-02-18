# AUDIO-025: Audio Asset Pipeline and Format Selection

**Category:** Audio / Technical
**Priority:** T4
**Source:** sound_design.md -- Section 8.5, 8.6

## Summary

Establish audio asset pipeline: OGG Vorbis for loops and ambient, WAV for short one-shots, streaming for long pieces (>30 seconds). Asset catalog with ~150-200 sound assets organized by category. Memory management: preload critical sounds, stream background, unload unused.

## Details

- OGG Vorbis: good compression, looping support, streaming capable
- WAV: zero-latency for responsive one-shots (tool sounds, stingers)
- Asset streaming for music stems (prevent memory bloat)
- Preload on scene load: tool sounds, notification sounds, common stingers
- Lazy load: seasonal beds, disaster sounds, rare events

## Dependencies

- AUDIO-001
- Bevy asset system

## Acceptance Criteria

- [ ] Audio assets organized by category
- [ ] Format selection per asset type
- [ ] Streaming for long audio
- [ ] Memory budget maintained
