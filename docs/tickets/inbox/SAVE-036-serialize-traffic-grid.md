# SAVE-036: Serialize Traffic Grid State

## Priority: T1 (Medium-Term)
## Effort: Small (0.5 day)
## Source: save_system_architecture.md -- Current Serialization Audit

## Description
Verify traffic density grid persists across save/load. Traffic overlay should show same patterns immediately after loading. Alternatively, rebuild from citizen positions on load if performance allows.

## Acceptance Criteria
- [ ] Traffic grid data serialized OR zeroed with fast rebuild
- [ ] Traffic overlay functional immediately after load
- [ ] No stale traffic data from previous sessions
