# CIT-078: Construction Material Requirements

**Priority:** T3 (Differentiation)
**Complexity:** Medium (2 person-weeks)
**Dependencies:** CIT-075 (production chains)
**Source:** master_architecture.md Section 1.4

## Description

Buildings require construction materials to build, not just time. Materials: concrete (from industry), steel (from ore), lumber (from forest). Material requirements increase with building level: L1 needs 100 concrete, L2 needs 200+100 steel, L5 needs 500+300+200. Materials sourced from local production or imported. Material shortage delays construction. This creates demand for industrial zone and resource extraction.

## Definition of Done

- [ ] Material requirements per building level
- [ ] Material sourcing from local production and imports
- [ ] Construction delay when materials unavailable
- [ ] Material cost added to construction budget
- [ ] Material stockpile tracking
- [ ] Construction material demand metric
- [ ] Material shortage notification

## Test Plan

- Unit test: building requires correct materials per level
- Unit test: missing materials delay construction
- Unit test: import fills material shortage at higher cost
- Integration test: industrial zone provides construction materials

## Pitfalls

- Material requirements shouldn't make early game impossible; low requirements for L1
- Must handle the case where player has no industry (import everything)

## Relevant Code

- `crates/simulation/src/buildings.rs` (progress_construction)
- `crates/simulation/src/production.rs`
