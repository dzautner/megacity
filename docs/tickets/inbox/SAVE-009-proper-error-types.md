# SAVE-009: Add Proper Error Types for Save/Load

## Priority: T1 (Short-Term Fix)
## Effort: Small (1 day)
## Source: save_system_architecture.md -- Future Architecture Recommendations

## Description
Errors are currently `eprintln!`'d and swallowed. Add proper `Result` propagation with a `SaveError` enum (IoError, DecodeError, VersionMismatch, ChecksumFailed, etc.).

## Acceptance Criteria
- [ ] `SaveError` enum with meaningful variants
- [ ] `handle_save` and `handle_load` return `Result<(), SaveError>`
- [ ] Errors displayed to user via UI notification
- [ ] No silent error swallowing
