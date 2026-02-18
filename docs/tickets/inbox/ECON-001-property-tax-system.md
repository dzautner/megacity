# ECON-001: Property Tax System (Replace Per-Citizen Flat Tax)
**Priority:** T1
**Complexity:** L
**Dependencies:** none
**Source:** economic_simulation.md, section 1.1; master_architecture.md, section M2

## Description
Replace the current per-citizen flat tax ($10 * tax_rate per citizen) with a property tax model based on building value and land value. This is the single most important economy change for M2 -- it connects land value to revenue and creates the core fiscal incentive loop.

- Property tax = (land_value + building_value) * tax_rate
- Land value from LandValueGrid for building's cell
- Building value = capacity_for_level * value_per_unit (varies by zone type)
- Tax collected per building, not per citizen
- Per-zone tax rates already exist in budget.rs ZoneTaxRates -- actually use them
- Tax revenue = sum of property_tax(building) for all buildings
- Commercial income remains separate (business revenue, not property)

## Definition of Done
- [ ] Tax calculated per building based on land value + building value
- [ ] Per-zone tax rates from ZoneTaxRates applied correctly
- [ ] collect_taxes uses property-based formula
- [ ] Budget income breakdown shows property tax vs commercial income
- [ ] Tax rate slider affects property tax income

## Test Plan
- Unit: Building in high land value area generates more tax than same building in low land value
- Unit: Higher zone tax rate produces proportionally more revenue
- Integration: Raise tax rate, verify income increases and happiness decreases
- Integration: Improve land value, verify tax revenue increases

## Pitfalls
- Must calibrate tax rates to maintain fiscal balance (too high = abandonment, too low = bankruptcy)
- The jump from flat tax to property tax changes total revenue -- needs careful tuning of rates
- ZoneTaxRates exists but is unused in collect_taxes -- wire it in
- Residential tax affects happiness differently than commercial tax

## Relevant Code
- `crates/simulation/src/economy.rs:collect_taxes` -- rewrite tax calculation
- `crates/simulation/src/budget.rs:ZoneTaxRates` -- already defined, needs usage
- `crates/simulation/src/land_value.rs:LandValueGrid` -- land value input
- `crates/simulation/src/buildings.rs:Building` -- building value calculation
