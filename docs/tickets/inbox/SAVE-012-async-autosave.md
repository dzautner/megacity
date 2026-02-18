# SAVE-012: Implement Async Autosave with Double-Buffered Snapshot

## Priority: T1 (Medium-Term)
## Effort: Medium (3-4 days)
## Source: save_system_architecture.md -- Autosave Design

## Description
Save is currently synchronous and blocks the main thread. Implement snapshot-then-serialize pipeline: snapshot world state into SaveData on main thread (<16ms), then encode/compress/write on background thread via `AsyncComputeTaskPool`.

## Acceptance Criteria
- [ ] Snapshot step completes within one frame (16ms for 100K citizens)
- [ ] Encoding, compression, and file writing happen on background thread
- [ ] `SaveInProgress` resource tracks active task
- [ ] `poll_save_completion` system checks task completion each frame
- [ ] UI shows "Saving..." indicator during background save
- [ ] No main-thread hitch for autosave
