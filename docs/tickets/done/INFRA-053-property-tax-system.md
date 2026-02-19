# INFRA-053: Property Tax Replacing Per-Citizen Flat Tax
**Priority:** T0
**Complexity:** M (1-3 days)
**Dependencies:** none
**Source:** master_architecture.md, M2; Section 6.2

## Description
Replace the current per-citizen flat tax with property tax based on land value and building level. Property tax revenue = sum of (land_value * building_level * tax_rate) for all buildings. Tax rate configurable by player (1-10%). Higher land value = higher tax revenue. Creates core feedback loop: services -> land value -> building upgrade -> more tax revenue. Current `collect_taxes` in `economy.rs` uses per-citizen rate.

## Definition of Done
- [ ] Property tax formula: land_value * building_area * tax_rate
- [ ] Tax rate configurable per zone type or district
- [ ] Revenue correctly computed from all buildings
- [ ] Per-citizen flat tax removed as primary revenue source
- [ ] Budget panel shows property tax as line item
- [ ] Tests pass

## Test Plan
- Unit: Building at land_value=100, level 3, rate 5% = expected revenue
- Unit: Doubling land value doubles property tax revenue
- Integration: Budget panel shows property tax replacing flat tax

## Pitfalls
- Removing flat tax changes game balance dramatically; may need transition period
- Very low land value areas generate nearly zero revenue; may need minimum
- Tax rate affects happiness (INFRA-073)

## Relevant Code
- `crates/simulation/src/economy.rs` -- `collect_taxes` function
- `crates/simulation/src/land_value.rs` -- land value per cell
- `crates/simulation/src/budget.rs` -- revenue categories
