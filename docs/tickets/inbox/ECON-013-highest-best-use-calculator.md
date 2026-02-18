# ECON-013: Highest and Best Use Calculator
**Priority:** T3
**Complexity:** L
**Dependencies:** ECON-006
**Source:** economic_simulation.md, section 2.6

## Description
Implement "highest and best use" (HBU) analysis that determines what zone type produces the maximum land value at each location. This enables zone recommendation UI and automatic zone suggestion overlays.

- For each cell, compute hypothetical land value under each zone type
- Zone-specific modifiers: Commercial values foot traffic + intersections, Office values CBD proximity + transit, ResidentialLow values schools + low crime + space, Industrial values highway + cheap land
- HBU overlay shows which zone type would maximize value at each cell
- Advisor system can recommend zone changes based on HBU analysis
- HBU differs from current zone by definition = "miszone indicator"

## Definition of Done
- [ ] HBU computed per cell across all zone types
- [ ] HBU overlay shows recommended zone type per cell
- [ ] Advisor suggests rezone when HBU differs from current zone
- [ ] Zone-specific value modifiers implemented

## Test Plan
- Unit: Cell near highway with no services: HBU = Industrial
- Unit: Cell near park + school + low density: HBU = ResidentialLow
- Integration: HBU overlay matches intuitive expectations for test city

## Pitfalls
- Expensive: computing HBU for 65K cells * 7 zone types each tick
- Zone type not matching HBU doesn't mean it's wrong (player intent matters)
- CBD proximity requires defining what the CBD is (highest land value cluster?)

## Relevant Code
- `crates/simulation/src/land_value.rs` -- zone-specific value computation
- `crates/simulation/src/advisors.rs` -- HBU-based recommendations
- `crates/rendering/src/overlay.rs` -- HBU overlay
