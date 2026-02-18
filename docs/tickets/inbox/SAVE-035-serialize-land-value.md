# SAVE-035: Serialize Land Value Grid

## Priority: T1 (Medium-Term)
## Effort: Small (0.5 day)
## Source: save_system_architecture.md -- Current Serialization Audit

## Description
Verify land value grid data persists across save/load. Land value is expensive to recompute and affects building upgrade decisions.

## Acceptance Criteria
- [ ] Land value grid data serialized
- [ ] Land value overlay matches pre-save state
- [ ] Building upgrade decisions consistent after load
