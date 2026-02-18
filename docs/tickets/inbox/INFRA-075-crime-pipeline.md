# INFRA-075: Crime Pipeline (Poverty -> Crime -> Policing -> Justice)
**Priority:** T2
**Complexity:** M (1-3 days)
**Dependencies:** INFRA-058
**Source:** master_architecture.md, M3

## Description
Implement full crime pipeline. Crime rate driven by: poverty, unemployment, low education, population density, low police coverage, low land value. Crime types: petty theft, burglary, assault. Police coverage reduces crime (not eliminates). Response time from dispatch (INFRA-058) affects arrest probability. Arrested criminals processed through justice system (simplified). High crime reduces land value and drives emigration.

## Definition of Done
- [ ] Crime generation from socioeconomic factors
- [ ] Multiple crime types with different severity
- [ ] Police effectiveness from coverage and response time
- [ ] Arrest probability based on response time
- [ ] Crime -> land value reduction feedback loop
- [ ] Crime overlay shows crime hotspots
- [ ] Tests pass

## Test Plan
- Unit: Poor area with no police has high crime rate
- Unit: Well-policed area has reduced crime
- Integration: Crime hotspots emerge organically from poverty clusters

## Pitfalls
- Crime-poverty feedback loop can create death spiral; need recovery mechanism
- Current `crime.rs` is basic; extend with new factors
- Must be careful about real-world sensitivity of crime modeling

## Relevant Code
- `crates/simulation/src/crime.rs` -- crime system
- `crates/simulation/src/happiness.rs` -- crime impact on happiness
- `crates/simulation/src/land_value.rs` -- crime impact on land value
