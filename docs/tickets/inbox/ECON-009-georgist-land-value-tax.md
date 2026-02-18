# ECON-009: Georgist Land Value Tax Option
**Priority:** T3
**Complexity:** M
**Dependencies:** ECON-001, ECON-006
**Source:** economic_simulation.md, section 2.7

## Description
Add Land Value Tax (LVT) as a policy option. Instead of taxing land + improvements (property tax), tax only the land component. Encourages development (empty lots pay same tax as buildings on same land) and discourages speculation.

- Policy toggle: switch from property tax to split-rate taxation
- LVT rate: 0-10% of land value (default 5%)
- Improvement tax rate: 0-5% of building value (default 2%, lower than standard property tax)
- Effects: empty/underbuilt lots face tax pressure to develop
- Building construction NOT penalized (encourages building)
- Modeled after Pittsburgh's split-rate tax experiment (1913-2001)

## Definition of Done
- [ ] LVT policy toggle in policies menu
- [ ] Tax calculation switches to land-only + reduced improvement rate
- [ ] Empty zoned lots generate tax expense (pressure to develop)
- [ ] Development rate increases when LVT enabled
- [ ] Revenue impact tracked in budget

## Test Plan
- Unit: LVT on empty lot = land_value * lvt_rate (no building component)
- Unit: LVT on developed lot = land_value * lvt_rate + building_value * improvement_rate
- Integration: Enable LVT, verify empty lots develop faster

## Pitfalls
- LVT on empty lots implies the lot has an "owner" paying tax -- city is the owner before development
- May need to apply as "development incentive" rather than true tax on unbuilt land
- Revenue calibration: LVT rate must generate comparable revenue to property tax

## Relevant Code
- `crates/simulation/src/economy.rs:collect_taxes` -- add LVT calculation path
- `crates/simulation/src/policies.rs` -- LVT toggle
- `crates/simulation/src/land_value.rs` -- land value component for tax
