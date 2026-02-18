# META-002: Save File Version Migration System

**Category:** Meta / Save System
**Priority:** T2
**Source:** master_architecture.md -- Section 1.17

## Summary

Version migration system with per-version migration functions. Delta/incremental saves. Autosave with configurable interval and slot rotation. Save file integrity checking (checksums). Save file compression (zstd). Serialize missing state (LifeSimTimer, PathCache, Velocity, VirtualPopulation).

## Details

- Save file version header
- Per-version migration functions (v1->v2, v2->v3, etc.)
- Autosave with 3-5 rotating slots
- Checksum validation on load
- zstd compression for smaller files
- Fix known serialization gaps

## Acceptance Criteria

- [ ] Save version header present
- [ ] Migration functions for version changes
- [ ] Autosave functional with rotation
- [ ] Missing state serialized (LifeSimTimer, PathCache, etc.)
