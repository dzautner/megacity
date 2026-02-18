# SAVE-032: Serialize Advisor Panel State

## Priority: T1 (Medium-Term)
## Effort: Small (0.5 day)
## Source: save_system_architecture.md -- Current Serialization Audit

## Description
Verify advisor panel state (dismissed tips, advisor history) persists across save/load.

## Acceptance Criteria
- [ ] Dismissed advisor tips stay dismissed
- [ ] Advisor history preserved
- [ ] No duplicate advisor messages after load
