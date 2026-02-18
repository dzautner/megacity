# POL-003: Schelling Segregation Model
**Priority:** T3
**Complexity:** M
**Dependencies:** ECON-012
**Source:** master_architecture.md, section M4

## Description
Implement Schelling's model of neighborhood segregation. Citizens have a tolerance threshold for neighbors of different income/education level. When threshold exceeded, they seek to move. This creates emergent neighborhood sorting without any scripted segregation.

- Each citizen has tolerance for income-different neighbors (0.3-0.8)
- Check: what fraction of building's residents differ significantly in income?
- If fraction > tolerance, citizen becomes "unhappy with neighborhood" and seeks to move
- Movement: citizen looks for building with more similar neighbors
- Result: neighborhoods naturally sort by income level over time
- Higher education increases tolerance (more diverse neighborhoods near universities)

## Definition of Done
- [ ] Citizens check neighbor similarity
- [ ] Unhappy citizens seek to relocate
- [ ] Neighborhoods naturally segregate by income
- [ ] Education affects tolerance
- [ ] Segregation visible in demographic overlay

## Test Plan
- Integration: Mixed-income area gradually sorts into income clusters
- Integration: Area near university remains more diverse

## Pitfalls
- Must not make segregation feel forced or political
- Schelling model is well-studied but parameters need game-specific tuning
- Must interact with housing filtering (ECON-010) and gentrification (ECON-017)

## Relevant Code
- `crates/simulation/src/citizen.rs:Personality` -- tolerance field
- `crates/simulation/src/movement.rs` -- relocation logic
- `crates/simulation/src/happiness.rs` -- neighborhood satisfaction component
