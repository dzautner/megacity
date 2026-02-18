# CIT-001: Extended Demographics Component

**Priority:** T3 (Differentiation)
**Complexity:** Medium (3-4 person-weeks)
**Dependencies:** None (extends existing CitizenDetails)
**Source:** social_agent_simulation.md Section 1.1, master_architecture.md Section 1.5

## Description

Extend `CitizenDetails` with a full `Demographics` component containing: ethnicity (EthnicGroup enum with 6 abstract groups), religion, income class (6 tiers from Poverty to Wealthy), occupation (Unemployed/BlueCollar/WhiteCollar/Professional/Executive/Retired), years of experience, marital status, household size, num_children, num_dependents, monthly income, monthly expenses, net worth, debt, and rent burden. These drive all downstream behavior: housing choice, job seeking, voting, happiness weighting, and segregation dynamics.

## Definition of Done

- [ ] `Demographics` component defined with all fields from research doc
- [ ] `EthnicGroup` enum with 6 abstract groups (GroupA-E, Mixed)
- [ ] `IncomeClass` enum with 6 tiers and methods for salary multipliers
- [ ] `Occupation` enum with 6 categories
- [ ] `MaritalStatus` enum (Single, Partnered, Married, Divorced, Widowed)
- [ ] Demographics populated at citizen spawn with distributions matching research doc tables
- [ ] Existing `CitizenDetails.education` (0-3) still works; Demographics provides extended 0-5
- [ ] Serialization support for all new enums and fields
- [ ] Backward-compatible save migration (old saves get default Demographics)

## Test Plan

- Unit test: distribution of spawned demographics matches target percentages within 5%
- Unit test: all enum variants serialize/deserialize correctly
- Integration test: 1000 citizens spawned, verify income distribution is log-normal
- Integration test: load old save without Demographics, verify defaults applied

## Pitfalls

- Adding too many components per citizen hurts ECS iteration performance; consider bundling related fields into a single component rather than multiple small ones
- The `education` field exists in both CitizenDetails (0-3) and Demographics (0-5); must keep them in sync or migrate fully

## Relevant Code

- `crates/simulation/src/citizen.rs` -- `CitizenDetails` struct (line 89-98)
- `crates/simulation/src/citizen_spawner.rs` -- citizen creation
- `crates/save/src/serialization.rs` -- save/load structs
