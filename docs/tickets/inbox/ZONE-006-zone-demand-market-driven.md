# ZONE-006: Market-Driven Zone Demand System
**Priority:** T1
**Complexity:** L
**Dependencies:** none
**Source:** urban_planning_zoning.md, section 3.3; master_architecture.md, section 1.3

## Description
Replace the current saturation-ratio demand system with a market-driven model. Current system: demand = base + pop_factor - saturation. Target: demand driven by employment ratio, commercial customer availability, housing vacancy rate, and trade connections.

- Residential demand = f(employment availability, immigration pressure, housing vacancy rate)
- Commercial demand = f(population spending power, foot traffic, goods supply)
- Industrial demand = f(trade connections, resource availability, labor supply)
- Office demand = f(educated workforce %, transit accessibility, land value)
- Track vacancy rates per zone type (built capacity vs occupied)
- Demand rises when vacancy < natural rate, falls when vacancy > natural rate
- Natural vacancy rates: Residential 5-7%, Commercial 5-8%, Industrial 5-8%, Office 8-12%

## Definition of Done
- [ ] Zone demand formula uses vacancy rates and market factors
- [ ] Vacancy rate calculated per zone type
- [ ] Demand responds to employment ratio, population, trade
- [ ] Natural vacancy rate equilibrium emerges
- [ ] Zone demand UI shows vacancy rates

## Test Plan
- Unit: With 0% vacancy, demand should be high
- Unit: With 20% vacancy, demand should be low/negative
- Integration: Build excess residential, verify demand drops below 0.2
- Integration: Add jobs (commercial/industrial), verify residential demand rises

## Pitfalls
- Must avoid oscillation (demand high -> build -> oversupply -> demand zero -> shortages)
- Damping factor needed to smooth demand changes over time
- Initial state (no buildings) needs bootstrapping demand

## Relevant Code
- `crates/simulation/src/zones.rs:update_zone_demand` -- rewrite demand formula
- `crates/simulation/src/zones.rs:ZoneDemand` -- add vacancy_rate fields
- `crates/simulation/src/buildings.rs:Building` -- occupants vs capacity for vacancy
