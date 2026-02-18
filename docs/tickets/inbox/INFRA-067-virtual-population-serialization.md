# INFRA-067: Fix VirtualPopulation Serialization
**Priority:** T0
**Complexity:** S (hours)
**Dependencies:** none
**Source:** master_architecture.md, M2; Section 5.1

## Description
Serialize the `VirtualPopulation` resource so that population scaling survives save/load. Currently virtual population data is lost on load, potentially causing population count discrepancies.

## Definition of Done
- [ ] VirtualPopulation resource serialized in save file
- [ ] Population count matches before and after save/load
- [ ] Tests pass

## Test Plan
- Unit: Save with 100K virtual + 10K real citizens, load, total matches
- Integration: Population counter shows same value after load

## Pitfalls
- VirtualPopulation may reference entity IDs that changed during deserialization
- Need to coordinate with citizen entity serialization

## Relevant Code
- `crates/simulation/src/virtual_population.rs` -- VirtualPopulation resource
- `crates/save/src/serialization.rs` -- save format
