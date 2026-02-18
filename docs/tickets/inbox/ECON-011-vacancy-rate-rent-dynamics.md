# ECON-011: Vacancy Rate and Rent Price Dynamics
**Priority:** T2
**Complexity:** M
**Dependencies:** ZONE-006
**Source:** economic_simulation.md, section 3.2

## Description
Implement vacancy rate tracking and rent/price index adjustment. Market equilibrium vacancy rates determine whether rents rise or fall. This is the primary price signal mechanism for the real estate market.

- Track per-zone: total_units, occupied_units, vacancy_rate
- Natural vacancy rates: Residential 5-7%, Commercial 5-8%, Industrial 5-8%, Office 8-12%
- Below natural rate: tight market, rents rise (up to +15%/year)
- Above natural rate: loose market, rents fall (up to -15%/year)
- Price indices (100=baseline): residential_price_index, commercial_rent_index, office_rent_index
- Price index affects citizen satisfaction, commercial profitability, immigration
- Price index clamped to 20-500 range

## Definition of Done
- [ ] Vacancy rate calculated per zone type
- [ ] Rent adjustment formula based on vacancy deviation from natural rate
- [ ] Price indices tracked over time
- [ ] Price changes visible in economy panel
- [ ] Price affects happiness, profitability, immigration

## Test Plan
- Unit: 0% vacancy produces positive rent adjustment
- Unit: 20% vacancy produces negative rent adjustment
- Integration: Build excess housing, verify price index drops

## Pitfalls
- Oscillation risk: overbuilding crashes prices -> construction stops -> prices spike -> overbuilding
- Need damping factor and update speed limits
- Price index affects both new construction feasibility and existing building revenue

## Relevant Code
- `crates/simulation/src/market.rs` -- housing market tracking (may already exist)
- `crates/simulation/src/zones.rs:ZoneDemand` -- vacancy feeds demand
- `crates/simulation/src/economy.rs` -- rent indices affect revenue
