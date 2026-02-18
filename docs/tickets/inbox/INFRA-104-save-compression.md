# INFRA-104: Advanced Save System (Compression, Cloud Save, Integrity)
**Priority:** T4
**Complexity:** M (1-3 days)
**Dependencies:** INFRA-063
**Source:** master_architecture.md, M5

## Description
Enhance save system with compression (LZ4 or zstd for fast compression), integrity checking (CRC32 checksum to detect corruption), and cloud save preparation (save file metadata for sync). Compressed save files should be 50-70% smaller than uncompressed. Integrity check on load warns about corrupted saves.

## Definition of Done
- [ ] Save file compression (LZ4 or zstd)
- [ ] CRC32 integrity checksum embedded in save
- [ ] Corruption detection on load with user warning
- [ ] Save file metadata (timestamp, city name, population, playtime)
- [ ] Compression reduces file size by 50%+
- [ ] Tests pass

## Test Plan
- Unit: Compressed save decompresses to identical data
- Unit: Modified save file triggers integrity warning
- Integration: Save/load cycle with compression works correctly

## Pitfalls
- Compression adds latency to save/load; LZ4 is fast enough for real-time
- Must handle both compressed and uncompressed saves during transition
- Cloud save API depends on platform (Steam, etc.)

## Relevant Code
- `crates/save/src/lib.rs` -- save/load pipeline
- `crates/save/src/serialization.rs` -- format
