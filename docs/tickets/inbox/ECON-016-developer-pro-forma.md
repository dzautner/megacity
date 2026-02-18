# ECON-016: Developer Pro Forma Feasibility Analysis
**Priority:** T3
**Complexity:** L
**Dependencies:** ECON-015, BLDG-008
**Source:** economic_simulation.md, section 3.4

## Description
Implement the developer pro forma model that determines if a building project is financially feasible. This is the economic engine behind market-driven development.

- DeveloperProForma struct: zone_type, land_cost, construction_cost_per_sqft, total_sqft, units, market_rent_per_sqft_month, vacancy_rate, operating_expense_ratio, cap_rate, financing_rate, construction_months
- Key metrics: Gross Potential Revenue, Effective Gross Income, Net Operating Income (NOI), Total Development Cost, Stabilized Value (NOI / cap_rate), Developer Profit, Yield on Cost
- Feasibility check: yield_on_cost > cap_rate + 1%, profit > 0, profit_margin > 10%
- Cap rates by property type: Multifamily A (4.5%), Office CBD (5.5%), Retail (6%), Industrial (5%)
- Used by BLDG-008 developer ROI model to determine what gets built

## Definition of Done
- [ ] DeveloperProForma calculation implemented
- [ ] All key metrics computed correctly
- [ ] Feasibility check gates building construction
- [ ] Cap rates defined per property type
- [ ] Pro forma results visible in developer info overlay

## Test Plan
- Unit: High rent + low construction cost = feasible
- Unit: Low rent + high construction cost = infeasible
- Unit: Yield on cost formula matches expected output

## Pitfalls
- Many interdependent parameters -- small changes cascade through all metrics
- Cap rates are external market data -- may need simplification for game context
- Operating expense ratio varies by building type (35-45% for residential, 20-30% for industrial)

## Relevant Code
- `crates/simulation/src/market.rs` -- pro forma calculation
- `crates/simulation/src/buildings.rs:building_spawner` -- feasibility gate
- `crates/simulation/src/economy.rs` -- market data inputs
