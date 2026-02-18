# ECON-019: Business Profitability and Closure Simulation
**Priority:** T3
**Complexity:** M
**Dependencies:** ECON-015, ECON-018
**Source:** economic_simulation.md, section 4; master_architecture.md, section 3

## Description
Model individual commercial/industrial/office buildings as businesses with profit/loss. When businesses are unprofitable (high rent + wages + tax > revenue), they close. Closure creates vacancies and job loss.

- Business revenue = customers * spending_per_visit (for commercial), production_output * price (for industrial)
- Business costs = rent + wages + utilities + tax
- Profit = revenue - costs
- Unprofitable businesses close after 6-12 game-months of losses
- Closure creates vacancies, unemployed workers, reduced demand
- Business opening: when vacant commercial lot + demand, new business starts (3-6 month startup)

## Definition of Done
- [ ] Commercial buildings have profit/loss calculation
- [ ] Unprofitable businesses close
- [ ] Closure creates vacancies and unemployment
- [ ] New businesses open in vacant lots when demand exists
- [ ] Business profitability visible in building info

## Test Plan
- Unit: High-traffic commercial building is profitable
- Unit: No-traffic high-rent building is unprofitable
- Integration: Raise taxes significantly, verify business closures increase

## Pitfalls
- Business profitability model must be simple enough to compute for thousands of buildings
- Cascade risk: business closure -> unemployment -> spending drop -> more closures
- Must not make all businesses close during economic downturns

## Relevant Code
- `crates/simulation/src/economy.rs` -- business profit calculation
- `crates/simulation/src/buildings.rs:Building` -- occupancy as business health indicator
- `crates/simulation/src/abandonment.rs` -- business closure path
