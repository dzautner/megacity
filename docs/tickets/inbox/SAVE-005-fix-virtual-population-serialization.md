# SAVE-005: Fix VirtualPopulation Serialization
**Priority:** T0
**Complexity:** S
**Dependencies:** none
**Source:** master_architecture.md, section M2 (known issue)

## Description
VirtualPopulation is not serialized, causing population count mismatch on load. Virtual citizens represent the scaled population beyond entity count (LOD abstraction).

- Serialize VirtualPopulation resource (virtual_count, demographic distribution)
- On load, restore virtual population to saved state
- Verify total population (entity + virtual) matches pre-save count

## Definition of Done
- [ ] VirtualPopulation serialized
- [ ] Population count matches after load
- [ ] Virtual demographic distribution preserved

## Test Plan
- Integration: Save city with 100K pop (many virtual), load, verify total pop matches

## Pitfalls
- virtual_population.rs may have fields that are hard to serialize
- Backward compatibility with saves that don't have virtual pop data

## Relevant Code
- `crates/simulation/src/virtual_population.rs` -- VirtualPopulation resource
- `crates/save/src/serialization.rs` -- add to save/load
