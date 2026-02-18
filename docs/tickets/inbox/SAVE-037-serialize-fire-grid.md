# SAVE-037: Serialize Fire Grid State

## Priority: T1 (Medium-Term)
## Effort: Small (0.5 day)
## Source: save_system_architecture.md -- Current Serialization Audit

## Description
Active fires should persist across save/load. A burning building saved during a fire should still be burning when loaded.

## Acceptance Criteria
- [ ] Active fire cells serialized
- [ ] Fire grid state roundtrips correctly
- [ ] Fire spread continues after load
