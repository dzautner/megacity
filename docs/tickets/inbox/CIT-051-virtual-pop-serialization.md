# CIT-051: Fix VirtualPopulation Serialization

**Priority:** T1 (Core -- bugfix)
**Complexity:** Low (0.25 person-weeks)
**Dependencies:** None
**Source:** master_architecture.md Section 1.17, MEMORY.md Known Issues

## Description

`VirtualPopulation.total_virtual` count is not serialized. On load, the virtual population count resets, causing the real-to-virtual ratio to be incorrect. Fix by adding total_virtual to SaveData. On load, restore the count. Adjust real_citizen_cap calculation on load to match restored ratio.

## Definition of Done

- [ ] VirtualPopulation.total_virtual added to SaveData
- [ ] Serialized on save, restored on load
- [ ] Real-to-virtual ratio correct after load
- [ ] Population display shows correct total after load

## Test Plan

- Unit test: VirtualPopulation count round-trips correctly
- Integration test: save 50K virtual pop -> load -> count is 50K

## Pitfalls

- Save migration: old saves without virtual pop data default to entity count

## Relevant Code

- `crates/simulation/src/virtual_population.rs` (VirtualPopulation)
- `crates/save/src/serialization.rs`
