# INFRA-124: Infrastructure Lifecycle Cost Display
**Priority:** T2
**Complexity:** S (hours)
**Dependencies:** none
**Source:** infrastructure_engineering.md, Section 10 (Lifecycle Costs)

## Description
Display total lifecycle cost for infrastructure before placement: construction (15-40%) + annual O&M (50-80%) + decommissioning (2-10%). Currently only construction cost is shown. Add estimated 20-year cost projection. This helps players understand true cost of building decisions. Construction is only 15-40% of total cost.

## Definition of Done
- [ ] Lifecycle cost computed: construction + 20 years of O&M + replacement
- [ ] Displayed in placement preview tooltip
- [ ] Shows both immediate cost and annual cost
- [ ] Displayed for roads, service buildings, utility infrastructure
- [ ] Tests pass

## Test Plan
- Unit: Road lifecycle cost = construction + 20 * annual_maintenance
- Unit: Lifecycle cost displayed matches manual calculation
- Integration: Player sees lifecycle cost when placing any infrastructure

## Pitfalls
- Lifecycle cost estimate is approximate; O&M costs may change
- Too much information in tooltip can overwhelm; use expandable detail

## Relevant Code
- `crates/rendering/src/input.rs` -- placement preview tooltip
- `crates/ui/src/info_panel.rs` -- building info display
