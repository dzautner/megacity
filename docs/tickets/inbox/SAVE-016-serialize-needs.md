# SAVE-016: Serialize Citizen Needs

## Priority: T1 (Medium-Term)
## Effort: Small (0.5 day)
## Source: save_system_architecture.md -- Issue 4: Needs Not Serialized

## Description
Needs (hunger, energy, social, entertainment) reset to default on load. Add needs fields to `SaveCitizen`.

## Acceptance Criteria
- [ ] `SaveCitizen` has hunger, energy, social, entertainment fields
- [ ] Needs roundtrip correctly
- [ ] Old saves use defaults
