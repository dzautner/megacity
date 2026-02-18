# SERV-006: Health and Disease Model
**Priority:** T2
**Complexity:** M
**Dependencies:** SERV-001
**Source:** cities_skylines_analysis.md, section 8; master_architecture.md, section 5.1

## Description
Expand health system from simple coverage radius to include disease mechanics, hospital capacity, ambulance dispatch, and health conditions from environmental factors.

- Health conditions: illness from pollution, water contamination, overcrowding, lack of heating
- Hospital types: clinic (small, 50 beds), hospital (medium, 200 beds), medical center (large, 500 beds)
- Sick citizens need hospital treatment -- dispatch ambulance, occupy bed
- Untreated illness: citizen dies (death care) or leaves city
- Public health: vaccination campaigns, clean water, pollution control reduce illness rate
- Health overlay: illness rate per area, hospital utilization

## Definition of Done
- [ ] Health conditions generated from environmental factors
- [ ] Hospital capacity tracked with bed utilization
- [ ] Ambulance dispatch for critical cases
- [ ] Untreated illness consequences
- [ ] Health statistics and overlay

## Test Plan
- Unit: Polluted area has higher illness rate
- Integration: Build hospital near polluted area, verify illness rate decreases
- Integration: Remove hospital, verify illness-related deaths increase

## Pitfalls
- health.rs exists with basic implementation
- Disease spread (epidemic) is T5 feature -- this is just environmental health
- Must balance: health problems should create interesting gameplay, not just frustration

## Relevant Code
- `crates/simulation/src/health.rs` -- expand health system
- `crates/simulation/src/pollution.rs` -- pollution as health input
- `crates/simulation/src/services.rs:ServiceType::Hospital` -- hospital capacity
