# ZONE-015: Per-District Zone Policies
**Priority:** T2
**Complexity:** M
**Dependencies:** none
**Source:** cities_skylines_analysis.md, section 12; master_architecture.md, section 2

## Description
Allow policies to be applied per-district instead of city-wide only. Each district can have different tax rates, building height limits, industrial bans, and service budget levels.

- Per-district tax rate overrides (currently ZoneTaxRates is city-wide)
- Per-district policies: high-rise ban, heavy traffic ban, small business, schools out, etc.
- District-specific industrial specializations (already partially in specialization.rs)
- Per-district service budget multiplier
- UI: district info panel shows active policies with toggle controls

## Definition of Done
- [ ] Tax rates settable per district
- [ ] At least 5 policies applicable per district
- [ ] Building systems respect district-level policies
- [ ] District info panel shows policy controls

## Test Plan
- Integration: Set high-rise ban in district A, verify level 4-5 buildings blocked in A but allowed in B
- Integration: Set higher tax in district, verify different revenue calculation

## Pitfalls
- Building upgrade needs district lookup for each building (potential performance concern)
- Must gracefully handle buildings not in any district (use city-wide defaults)
- UI complexity -- too many per-district controls can overwhelm

## Relevant Code
- `crates/simulation/src/districts.rs:DistrictPolicies` -- expand per-district policy set
- `crates/simulation/src/budget.rs:ZoneTaxRates` -- per-district override
- `crates/simulation/src/building_upgrade.rs` -- check district policies
- `crates/ui/src/info_panel.rs` -- district policy UI
