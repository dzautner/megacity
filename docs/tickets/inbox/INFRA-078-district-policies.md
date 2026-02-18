# INFRA-078: District-Level Policies
**Priority:** T2
**Complexity:** M (1-3 days)
**Dependencies:** none
**Source:** master_architecture.md, M3

## Description
Allow per-district policy settings: tax rates, service spending levels, parking policies, zoning restrictions, truck bans. Currently districts track statistics but do not support per-district policies. Player paints districts and configures policies for each. Policies in one district do not affect others.

## Definition of Done
- [ ] District policy struct with configurable settings
- [ ] Per-district tax rate override
- [ ] Per-district service spending modifier
- [ ] Per-district zoning restrictions (e.g., no industrial in this district)
- [ ] District policy UI panel
- [ ] Tests pass

## Test Plan
- Unit: Tax rate override in district applies to buildings in that district only
- Unit: Truck ban policy prevents truck routing through district roads
- Integration: Creating a low-tax business district attracts commercial growth

## Pitfalls
- District boundaries may change; policies must follow the cells, not the shape
- Default district (cells not in any named district) needs default policies
- Too many policy options overwhelms player; start with tax rate and basic restrictions

## Relevant Code
- `crates/simulation/src/districts.rs` -- district system
- `crates/simulation/src/policies.rs` -- policy framework
- `crates/simulation/src/economy.rs` -- per-district tax collection
