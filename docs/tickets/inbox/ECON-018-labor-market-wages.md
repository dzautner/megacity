# ECON-018: Labor Market and Wage Determination
**Priority:** T3
**Complexity:** L
**Dependencies:** ECON-012
**Source:** economic_simulation.md, section 4; master_architecture.md, section 3

## Description
Implement labor market dynamics where wages are determined by supply and demand for workers at each education level. Tight labor markets (many jobs, few workers) raise wages. Loose markets (few jobs, many workers) lower wages.

- Track workers by education: uneducated, elementary, high school, university
- Track jobs by required education per zone type
- Wage = base_wage * supply_demand_factor
- supply_demand_factor: if workers < jobs, factor > 1 (wages rise); if workers > jobs, factor < 1
- Wages affect: citizen happiness, commercial profitability, immigration attractiveness
- Education pipeline output creates future labor supply
- Minimum wage policy option

## Definition of Done
- [ ] Labor supply tracked by education level
- [ ] Job demand tracked by education level
- [ ] Wages computed from supply/demand ratio
- [ ] Wages affect happiness and immigration
- [ ] Minimum wage policy available

## Test Plan
- Unit: Worker shortage (supply < demand) increases wages
- Unit: Worker surplus decreases wages
- Integration: Build university, verify educated workforce expands, wages for educated decline

## Pitfalls
- Education pipeline has latency (university takes time) -- must avoid oscillation
- Wage floor needed to prevent exploitation spiral
- Different zone types require different education mixes

## Relevant Code
- `crates/simulation/src/education_jobs.rs` -- education tracking
- `crates/simulation/src/economy.rs` -- wage computation
- `crates/simulation/src/happiness.rs` -- wage satisfaction component
- `crates/simulation/src/immigration.rs` -- wage attractiveness
