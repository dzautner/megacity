# MOD-004: Extract Policy Effects to Data Files

## Priority: T2 (Depth)
## Effort: Medium (2-3 days)
## Source: modding_architecture.md -- Data-Driven Architecture

## Description
Policy effects (tax modifiers, environmental rules, building permissions) are hardcoded. Extract to data files with effect descriptors.

## Acceptance Criteria
- [ ] `PolicyDef` struct: name, description, effects (list of modifier-target pairs), cost
- [ ] `assets/data/policies.ron` with all policy definitions
- [ ] Policy system reads effects from data file
- [ ] Modders can add new policies via data files
