# BLDG-008: Developer ROI Growth Model
**Priority:** T3
**Complexity:** XL
**Dependencies:** BLDG-005, BLDG-006, ECON-006
**Source:** urban_planning_zoning.md, section 3.3; economic_simulation.md, section 3.4

## Description
Replace demand-based building spawning with a market-driven developer ROI model. Instead of "demand exists, building appears," evaluate each vacant lot's financial feasibility: construction_cost vs expected_revenue. Only build when ROI > minimum threshold (8-15%).

- For each vacant zoned cell: estimate land value, expected occupancy, rent per unit
- For each allowable building type: compute construction_cost, expected_revenue over 10 years, ROI
- Select building with highest adjusted_roi (risk-adjusted)
- Only spawn if adjusted_roi > 0.10 (10% minimum return)
- Existing buildings: evaluate redevelopment (BLDG-005) via same ROI model
- This replaces the demand-based spawn chance system

## Definition of Done
- [ ] DeveloperDecision struct computed per vacant lot
- [ ] ROI calculation uses land value, demand, rent, construction cost
- [ ] Buildings only spawn when financially feasible
- [ ] Building type selection optimizes for highest ROI
- [ ] Vacant lots remain empty when development is unprofitable

## Test Plan
- Unit: High land value + high demand = high ROI = building spawns
- Unit: Low demand + high construction cost = low ROI = no building
- Integration: Area with no demand remains vacant despite being zoned

## Pitfalls
- Rent and construction cost need realistic calibration to prevent all/no buildings
- Must bootstrap initial development (early city has low demand but needs buildings)
- Performance: evaluating every vacant cell every tick is expensive; sample subset
- Player may not understand why zoned areas don't develop (need UI feedback)

## Relevant Code
- `crates/simulation/src/buildings.rs:building_spawner` -- replace with ROI evaluation
- `crates/simulation/src/land_value.rs` -- land value as input
- `crates/simulation/src/zones.rs:ZoneDemand` -- demand as occupancy predictor
- `crates/simulation/src/economy.rs` -- construction cost deduction
