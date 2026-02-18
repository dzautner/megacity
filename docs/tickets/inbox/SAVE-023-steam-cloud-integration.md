# SAVE-023: Steam Cloud Save Integration

## Priority: T4 (Polish)
## Effort: Medium (3-5 days)
## Source: save_system_architecture.md -- Cloud Save, master_architecture.md T4

## Description
Integrate with Steam Cloud for automatic save synchronization across machines. Use Steamworks API for cloud storage. Ensure compressed save sizes are within Steam Cloud quotas.

## Acceptance Criteria
- [ ] Save files synced via Steam Cloud API
- [ ] Conflict resolution when local and cloud saves diverge
- [ ] Save size optimized to fit within Steam Cloud quotas
- [ ] Toggle in settings to enable/disable cloud saves
- [ ] Graceful fallback when offline
