# ECON-015: Commercial Rent per Square Foot Model
**Priority:** T3
**Complexity:** M
**Dependencies:** ECON-011
**Source:** economic_simulation.md, section 3.3

## Description
Implement realistic commercial rent pricing based on location type and building class. Retail rent varies from $8/sqft (strip mall) to $200/sqft (urban high street). Office rent varies from $12/sqft (suburban Class B) to $100+/sqft (premium CBD).

- Commercial buildings generate revenue = floor_area * rent_per_sqft * occupancy
- Rent per sqft determined by: zone type, land value, building level (class), foot traffic
- Retail rent tiers: Strip ($15), Neighborhood ($22), Urban ($75)
- Office rent tiers: Suburban B ($20), CBD A ($55), Trophy ($90)
- Industrial rent: Warehouse ($6), Distribution ($10), Light Industrial ($12)
- Rent feeds into tax revenue and developer ROI calculations

## Definition of Done
- [ ] Per-sqft rent computed for all commercial/office/industrial buildings
- [ ] Rent varies by location quality and building class
- [ ] Revenue = area * rent * occupancy
- [ ] Rent affects building profitability and tax revenue

## Test Plan
- Unit: CBD office building rent > suburban office rent
- Unit: Empty building generates zero revenue
- Integration: Build commercial district, verify rent-based revenue calculations

## Pitfalls
- Need to define "floor area" per building (currently only capacity exists)
- Rent model interacts with vacancy/price dynamics -- must not double-count
- Calibration: rent * occupancy * tax must produce viable city budgets

## Relevant Code
- `crates/simulation/src/economy.rs:collect_taxes` -- use rent model for commercial income
- `crates/simulation/src/buildings.rs:Building` -- add floor_area field or compute from capacity
- `crates/simulation/src/market.rs` -- rent tracking
