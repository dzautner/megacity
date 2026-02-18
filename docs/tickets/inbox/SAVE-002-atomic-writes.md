# SAVE-002: Implement Atomic Writes (Write-Rename Pattern)

## Priority: T1 (Short-Term Fix)
## Effort: Small (0.5 day)
## Source: save_system_architecture.md -- Save File Security and Integrity

## Description
The current `std::fs::write` is not atomic. If the process crashes during write, the save file is corrupted and the previous save is destroyed. Implement write-rename pattern: write to `{path}.tmp`, call `sync_all()`, then atomically rename.

## Current Behavior
```rust
std::fs::write(&path, &bytes)
```
A crash during write corrupts the file.

## Acceptance Criteria
- [ ] Saves write to a `.tmp` file first
- [ ] `sync_all()` called before rename
- [ ] Atomic rename replaces the final file
- [ ] Previous save survives a crash during write
- [ ] Works on both POSIX and Windows

## Files
- `crates/save/src/lib.rs` (handle_save)
