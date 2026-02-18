# ENV-004: Soil Contamination from Industrial Land Use
**Priority:** T2
**Complexity:** M
**Dependencies:** none
**Source:** master_architecture.md, section M3

## Description
Industrial zones should contaminate soil over time. Contamination persists even after industrial buildings are removed (brownfield sites). Remediation is expensive. This creates realistic urban redevelopment constraints.

- Industrial buildings accumulate soil contamination per game-year
- Contamination persists when building demolished (stored on cell)
- Contaminated cells: -30% land value, health risk, cannot be rezoned residential without remediation
- Remediation: expensive ($5000/cell), takes 6-12 game-months
- Brownfield redevelopment: policy incentive to clean and reuse contaminated land
- Contamination visible in pollution overlay (distinct from air pollution)

## Definition of Done
- [ ] Soil contamination accumulates under industrial buildings
- [ ] Contamination persists after demolition
- [ ] Contaminated cells affect land value and health
- [ ] Remediation mechanic available
- [ ] Visible in pollution overlay

## Test Plan
- Integration: Build industrial for 20 years, demolish, verify contamination persists
- Integration: Remediate cell, verify land value recovers

## Pitfalls
- water_pollution.rs handles water contamination; this is soil (separate)
- Contamination should not spread (unlike water pollution)
- Remediation cost must be balanced against redevelopment value

## Relevant Code
- `crates/simulation/src/pollution.rs` -- soil contamination layer
- `crates/simulation/src/grid.rs:Cell` -- add contamination field
- `crates/simulation/src/land_value.rs` -- contamination penalty
