# CIT-056: Real Estate Market (Housing Prices)

**Priority:** T3 (Differentiation)
**Complexity:** Medium (3-4 person-weeks)
**Dependencies:** CIT-052 (property tax)
**Source:** master_architecture.md Section 1.8

## Description

Real estate market where housing prices emerge from supply/demand. Housing price = base_construction_cost * land_value_multiplier * demand_factor. Rent = housing_price * annual_rate / 12. Supply = total dwelling units. Demand = households seeking housing. When demand > supply, prices rise. When supply > demand, prices fall. Housing price index tracked over time. Speculation: land near planned transit increases in value before construction.

## Definition of Done

- [ ] Per-building housing price calculation
- [ ] Rent derived from housing price
- [ ] Supply/demand housing price adjustment
- [ ] Housing price index metric
- [ ] Historical price tracking (graph)
- [ ] Speculation mechanic for planned infrastructure
- [ ] Price affects affordability and rent burden
- [ ] Real estate stats in economy panel

## Test Plan

- Unit test: housing shortage raises prices
- Unit test: overbuilding reduces prices
- Unit test: planned transit increases nearby prices
- Integration test: housing price index visible over time

## Pitfalls

- Price volatility should be dampened (10-20% per year max change)
- Must connect to rent burden / affordability crisis systems

## Relevant Code

- `crates/simulation/src/buildings.rs`
- `crates/simulation/src/land_value.rs`
- `crates/simulation/src/economy.rs`
