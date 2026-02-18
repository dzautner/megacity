# SAVE-001: Add File Header with Magic Bytes, Version, and Checksum

## Priority: T1 (Short-Term Fix)
## Effort: Small (1-2 days)
## Source: save_system_architecture.md -- Future Architecture Recommendations, Save File Format Design

## Description
The current save format is a raw bitcode blob with no header. Add a file header containing:
- Magic bytes: "MEGA" (0x4D454741)
- Format version (u32, little-endian)
- Flags (u32: bit 0 = compressed, bit 1 = delta save)
- Timestamp (Unix epoch, u64)
- Uncompressed data size (u32)
- Checksum of compressed data (xxHash32)

## Current Behavior
No header, no version number, no magic bytes, no checksum. Old saves silently fail to decode when `SaveData` struct changes.

## Acceptance Criteria
- [ ] Save files start with "MEGA" magic bytes
- [ ] Format version u32 is written and read
- [ ] xxHash32 checksum verifies data integrity on load
- [ ] Corrupted files produce a clear error message
- [ ] Future saves from newer game versions are detected and rejected gracefully

## Files
- `crates/save/src/lib.rs`
- `crates/save/src/serialization.rs`
