# SAVE-021: Serialize Virtual Population Count

## Priority: T1 (Short-Term Fix)
## Effort: Small (0.5 day)
## Source: save_system_architecture.md -- Issue 5: VirtualPopulation Not Serialized

## Description
`VirtualPopulation::count` resets to 0 on load. A city with 500K virtual population loses them all. Add `virtual_population_count: u32` to `SaveData`.

## Acceptance Criteria
- [ ] `SaveData` includes `virtual_population_count`
- [ ] Virtual population roundtrips correctly
- [ ] Old saves default to 0
