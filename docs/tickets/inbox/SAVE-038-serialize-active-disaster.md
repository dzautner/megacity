# SAVE-038: Serialize Active Disaster State

## Priority: T1 (Medium-Term)
## Effort: Small (0.5 day)
## Source: save_system_architecture.md -- Current Serialization Audit

## Description
If a disaster is in progress when saving, it should resume on load with remaining duration and affected area.

## Acceptance Criteria
- [ ] ActiveDisaster resource serialized
- [ ] Disaster type, position, radius, remaining_ticks preserved
- [ ] Disaster continues after load
