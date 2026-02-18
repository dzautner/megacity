# SAVE-027: Serialize Event Journal and Active Effects

## Priority: T1 (Medium-Term)
## Effort: Small (1 day)
## Source: save_system_architecture.md -- Current Serialization Audit

## Description
The EventJournal and ActiveCityEffects may not be fully serialized. Verify and fix so historical events and active modifiers persist across save/load.

## Acceptance Criteria
- [ ] EventJournal entries roundtrip correctly
- [ ] ActiveCityEffects (temporary modifiers) roundtrip
- [ ] MilestoneTracker state roundtrips
- [ ] Event history visible after load
