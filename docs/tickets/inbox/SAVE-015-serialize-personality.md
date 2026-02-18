# SAVE-015: Serialize Citizen Personality

## Priority: T1 (Medium-Term)
## Effort: Small (0.5 day)
## Source: save_system_architecture.md -- Issue 4: Personality Not Serialized

## Description
Personality (ambition, sociability, materialism, resilience) is hardcoded to 0.5 on load, making all citizens identical. Add personality fields to `SaveCitizen`.

## Acceptance Criteria
- [ ] `SaveCitizen` has ambition, sociability, materialism, resilience fields
- [ ] Personality roundtrips correctly
- [ ] Old saves default to 0.5
