# SAVE-034: Serialize Pollution Grid State

## Priority: T1 (Medium-Term)
## Effort: Small (0.5 day)
## Source: save_system_architecture.md -- Current Serialization Audit

## Description
Verify pollution grid data persists across save/load. Air, noise, and water pollution grids should all roundtrip.

## Acceptance Criteria
- [ ] Air pollution grid serialized
- [ ] Noise pollution grid serialized
- [ ] Water pollution grid serialized
- [ ] Pollution overlay matches pre-save state
