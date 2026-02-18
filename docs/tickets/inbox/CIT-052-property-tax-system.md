# CIT-052: Property Tax System

**Priority:** T2 (Depth)
**Complexity:** Medium (3-4 person-weeks)
**Dependencies:** None
**Source:** master_architecture.md Section 1.8, historical_demographics_services.md Section 6.5

## Description

Replace per-citizen flat tax with property tax on assessed building value. Property tax = assessed_value * millage_rate. Assessed value = land_value * lot_size * building_level_multiplier * assessment_ratio. Millage rate adjustable per zone type (residential, commercial, industrial). Tax increment financing (TIF) districts redirect property tax growth to infrastructure bonds. Property tax is the primary revenue source (30-45% of budget).

## Definition of Done

- [ ] Per-building assessed value calculation
- [ ] Millage rate per zone type (adjustable by player)
- [ ] Property tax revenue = sum(assessed_value * millage_rate) for all buildings
- [ ] Assessment ratio (fraction of market value taxed)
- [ ] Tax displayed per building in inspection panel
- [ ] Tax revenue replaces per-citizen flat tax
- [ ] TIF district mechanic (redirect tax growth to area infrastructure)
- [ ] Property tax as 30-45% of total revenue

## Test Plan

- Unit test: high land value building pays more tax
- Unit test: millage rate changes affect revenue linearly
- Unit test: TIF district redirects growth revenue
- Integration test: property tax revenue scales with city growth

## Pitfalls

- Transition from per-citizen to per-building tax changes revenue dynamics significantly
- Must handle buildings with zero occupants (still pay property tax)

## Relevant Code

- `crates/simulation/src/economy.rs` (collect_taxes)
- `crates/simulation/src/budget.rs` (ZoneTaxRates)
- `crates/simulation/src/land_value.rs`
