# AUDIO-029: Chunk-Based Audio Aggregation

**Category:** Audio / Technical
**Priority:** T4
**Source:** sound_design.md -- Section 2.7

## Summary

Instead of individual audio emitters per cell, aggregate audio per chunk (8x8). Each chunk tracks dominant zone, traffic density, building density, water presence. Single emitter per chunk with pre-mixed sound for that chunk's composition. Massively reduces audio voice count.

## Acceptance Criteria

- [ ] Audio aggregation per 8x8 chunk
- [ ] Chunk composition drives ambient mix
- [ ] Voice count within budget (32-48)
- [ ] No audible difference from per-cell at normal zoom
