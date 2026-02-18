# SAVE-011: Add LZ4 Compression for All Saves

## Priority: T1 (Medium-Term)
## Effort: Small (1 day)
## Source: save_system_architecture.md -- Compression

## Description
Add LZ4 compression to reduce save file size by 2-3x. Use `lz4_flex` crate. A 50MB uncompressed save becomes ~15MB.

## Acceptance Criteria
- [ ] `lz4_flex` added as dependency
- [ ] Save data compressed before writing
- [ ] Load decompresses before decoding
- [ ] Header flags bit indicates compression
- [ ] Benchmark: compression adds <10% to save time
