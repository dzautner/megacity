# WASTE-011: Waste Management Dashboard UI Panel

## Priority: T1 (Core)

## Description
Create a waste management dashboard showing total generation, collection rate, diversion rate (recycling + composting), landfill remaining capacity, and waste stream breakdown. This is the player's interface for managing waste infrastructure.

## Current State
- No waste-related UI panel.
- No waste metrics displayed.

## Definition of Done
- [ ] Dashboard showing: total waste generated (tons/day), collected (tons/day), uncollected (tons).
- [ ] Diversion metrics: recycling rate (%), composting rate (%), WTE rate (%).
- [ ] Landfill capacity: current fill (%), years remaining estimate.
- [ ] Waste stream pie chart: paper, food, yard, plastics, metals, glass, wood, textiles, other.
- [ ] Collection coverage: % of buildings served.
- [ ] Monthly waste budget: collection cost, processing cost, recycling revenue, net cost.
- [ ] Warning indicators for: low landfill capacity, uncollected waste, overflow.

## Test Plan
- [ ] UI test: dashboard shows correct waste generation rate.
- [ ] UI test: landfill capacity bar fills over time.
- [ ] UI test: warning appears when landfill below 25%.

## Pitfalls
- Requires WASTE-001 and WASTE-003 to have meaningful data to display.
- Waste stream breakdown requires WASTE-002 (composition model).
- Monthly budget needs integration with economy system.

## Code References
- `crates/ui/src/info_panel.rs`: existing panel system
