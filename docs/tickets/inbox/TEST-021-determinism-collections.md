# TEST-021: Use Deterministic Collections for Iteration-Order Dependent Code

## Priority: T2 (Depth)
## Effort: Medium (2-3 days)
## Source: testing_strategy.md -- Section 4.2: HashMap Non-Determinism

## Description
Audit all HashMap usage in simulation crate. Replace with BTreeMap or sort results where iteration order affects simulation output. RoadNetwork::edges is primary target.

## Acceptance Criteria
- [ ] Audit all HashMap usage in simulation crate
- [ ] RoadNetwork::edges: replace HashMap or sort after iteration
- [ ] Temporary HashMaps in systems: sort results
- [ ] Document which collections are order-dependent
