# INFRA-129: Demand Response Policy for Power Grid
**Priority:** T3
**Complexity:** S (hours)
**Dependencies:** INFRA-119
**Source:** infrastructure_engineering.md, Section 7 (Peak Demand)

## Description
Implement demand response as a policy that reduces peak electricity demand by 5-15% without building peaker plants. Options: time-of-use pricing (electricity costs more during peaks), commercial load shedding (HVAC reduction), smart thermostat programs. Requires smart grid infrastructure investment. Cheaper than building peaker plants. A 1% shift in peak demand saves ~3.9% in system costs.

## Definition of Done
- [ ] Demand response policy toggle
- [ ] Smart grid infrastructure requirement
- [ ] Peak demand reduction of 5-15%
- [ ] Reduced need for peaker plants
- [ ] Time-of-use pricing option
- [ ] Tests pass

## Test Plan
- Unit: Demand response reduces peak demand by 10%
- Unit: Without smart grid, demand response not available
- Integration: Demand response prevents brownout that would otherwise occur

## Pitfalls
- Demand response effectiveness degrades during extended heatwaves
- Citizen satisfaction impact from load shedding/higher peak prices
- Smart grid investment cost must be balanced against peaker plant alternative

## Relevant Code
- `crates/simulation/src/policies.rs` -- demand response policy
- `crates/simulation/src/utilities.rs` -- peak demand calculation
