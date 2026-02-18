# SAVE-019: Crash Recovery Detection

## Priority: T1 (Medium-Term)
## Effort: Small (1 day)
## Source: save_system_architecture.md -- Crash Recovery

## Description
On startup, check for `.tmp` files (incomplete writes from crashes). Iterate autosave slots in reverse order, verify checksums, and prompt player to load the most recent valid autosave.

## Acceptance Criteria
- [ ] `.tmp` files detected and cleaned up on startup
- [ ] Autosave slots validated in reverse order
- [ ] Checksum verification for each slot
- [ ] Dialog prompt: "A crash was detected. Load the most recent autosave?"
- [ ] First valid autosave offered for recovery
