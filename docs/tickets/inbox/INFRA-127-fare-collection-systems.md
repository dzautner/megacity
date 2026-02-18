# INFRA-127: Fare Collection Systems and Pricing
**Priority:** T2
**Complexity:** S (hours)
**Dependencies:** INFRA-037
**Source:** infrastructure_engineering.md, Section 2 (Fare Collection)

## Description
Implement fare collection policy options for transit. Types: flat fare ($1-3 per trip), distance-based, zone-based (city divided into fare zones), time-based (day passes, monthly passes), free transit. Each has ridership and revenue implications. Fare elasticity: 1% fare increase reduces ridership by ~0.4%. Employer subsidies and monthly passes boost ridership.

## Definition of Done
- [ ] `FarePolicy` enum with fare types
- [ ] Fare affects mode choice utility
- [ ] Revenue computation per fare type
- [ ] Fare elasticity reducing ridership at higher fares
- [ ] Monthly pass option (flat cost, unlimited rides)
- [ ] Free transit policy option
- [ ] Tests pass

## Test Plan
- Unit: 10% fare increase reduces ridership by ~4%
- Unit: Free transit maximizes ridership but zero fare revenue
- Integration: Player adjusts fares to balance ridership and revenue

## Pitfalls
- Free transit is popular but eliminates fare revenue (needs tax subsidy)
- Zone-based fares need geographic zone definition
- Distance-based fares need per-trip distance tracking

## Relevant Code
- `crates/simulation/src/policies.rs` -- fare policy
- `crates/simulation/src/economy.rs` -- fare revenue
