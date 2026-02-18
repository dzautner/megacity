# ECON-020: Real Estate Market Resource
**Priority:** T3
**Complexity:** M
**Dependencies:** ECON-011, ECON-015
**Source:** economic_simulation.md, section 3.2; master_architecture.md, section 3

## Description
Create a centralized HousingMarket resource that tracks aggregate market statistics: total units, occupied units, vacancy rates, price indices, construction pipeline. This is the single source of truth for market conditions.

- HousingMarket fields: total_residential_units, occupied_residential_units, total_commercial_sqft, occupied_commercial_sqft, total_office_sqft, occupied_office_sqft
- Derived: residential_vacancy, commercial_vacancy, office_vacancy
- Price indices: residential_price_index (100=baseline), commercial_rent_index, office_rent_index
- Update every slow tick from building query
- Displayed in economy panel with trend graphs

## Definition of Done
- [ ] HousingMarket resource with all fields
- [ ] Updated from building queries each slow tick
- [ ] Price indices adjust based on vacancy
- [ ] Economy panel displays market statistics
- [ ] Historical trend data stored for graphs

## Test Plan
- Unit: Total units = sum of all building capacities
- Unit: Vacancy = 1 - (occupied/total)
- Integration: Build housing, verify total units increases, vacancy adjusts

## Pitfalls
- Must handle zero-buildings case (no division by zero)
- Price index history needs bounded storage (ring buffer of last 100 data points)
- market.rs may already have partial implementation

## Relevant Code
- `crates/simulation/src/market.rs` -- expand or create HousingMarket resource
- `crates/simulation/src/buildings.rs:Building` -- data source for aggregation
- `crates/ui/src/info_panel.rs` -- market statistics display
