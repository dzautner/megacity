# INFRA-084: Housing Affordability Crisis and Gentrification
**Priority:** T3
**Complexity:** L (3-5 days)
**Dependencies:** INFRA-053, INFRA-060
**Source:** master_architecture.md, M4

## Description
Model housing affordability: rent proportional to land value and building quality. When rent exceeds citizen income threshold (~30%), citizens are "cost-burdened." Rising land values cause gentrification: low-income residents displaced as rents increase. Displacement tracked as emigration events. Policy tools: rent control, affordable housing mandates, inclusionary zoning. Housing affordability index displayed in stats.

## Definition of Done
- [ ] Rent calculation from land value and building level
- [ ] Cost-burden detection (rent > 30% income)
- [ ] Gentrification displacement: cost-burdened citizens seek cheaper housing
- [ ] Housing affordability index metric
- [ ] Policy tools: rent control, affordable housing mandates
- [ ] Tests pass

## Test Plan
- Unit: Citizen with $3000 income in $1200/month apartment is cost-burdened
- Unit: Land value increase raises rents, displaces low-income residents
- Integration: Downtown gentrification visible as low-income areas turn high-income

## Pitfalls
- Rent control has known unintended consequences (reduced construction); model these
- Need income distribution across citizens, not flat income
- Gentrification is politically sensitive; present neutrally with tradeoffs

## Relevant Code
- `crates/simulation/src/wealth.rs` -- citizen income
- `crates/simulation/src/land_value.rs` -- rent calculation basis
- `crates/simulation/src/homelessness.rs` -- displacement outcome
