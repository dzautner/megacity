# CIT-075: Production Chain Enhancement

**Priority:** T3 (Differentiation)
**Complexity:** High (4-5 person-weeks)
**Dependencies:** None
**Source:** master_architecture.md Section 1.8

## Description

Deeper commodity production chains. Raw materials (from resource extraction) -> processed goods (factories) -> finished products (advanced industry) -> retail (commercial). Chain: Forest -> Lumber -> Furniture -> Furniture Store. Oil -> Petroleum -> Plastics -> Consumer Goods. Ore -> Steel -> Construction Materials -> Buildings. Each chain step requires appropriate industrial building. Missing chain links = import dependency (cost). Complete local chains = economic bonus.

## Definition of Done

- [ ] Extended commodity types (raw, processed, finished)
- [ ] Production chain definitions (input -> output mapping)
- [ ] Industrial building specialization by chain step
- [ ] Import/export for missing chain links
- [ ] Local production discount vs import cost
- [ ] Production chain visualization (flow diagram)
- [ ] Employment by chain step
- [ ] Resource depletion affects chain input

## Test Plan

- Unit test: factory produces output from inputs
- Unit test: missing input triggers import
- Unit test: complete local chain provides cost bonus
- Integration test: resource city develops downstream industry

## Pitfalls

- Production chains add significant complexity; keep simple initially
- Existing production.rs has basic goods; extend, don't replace

## Relevant Code

- `crates/simulation/src/production.rs` (CityGoods, update_production_chains)
- `crates/simulation/src/natural_resources.rs` (ResourceGrid)
- `crates/simulation/src/market.rs` (MarketPrices)
