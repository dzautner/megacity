# SERV-005: Crime Types and Justice Pipeline
**Priority:** T2
**Complexity:** L
**Dependencies:** SERV-001
**Source:** cities_skylines_analysis.md, section 5; master_architecture.md, section M3

## Description
Expand crime from simple crime rate to a pipeline: conditions generate crime events, police respond, criminals are arrested, tried, and imprisoned. Different crime types have different effects and require different responses.

- Crime types: petty theft, vandalism, burglary, assault, organized crime
- Crime generation: poverty + low education + low police coverage + unemployment
- Police response: dispatch officers, response time affects arrest probability
- Arrest -> court system (simplified) -> prison (if convicted)
- Prison capacity: too few cells = criminals released early = recidivism
- Organized crime: emerges in high-crime areas, harder to combat, requires detective units
- Crime affects: happiness, land value, business closures, tourism

## Definition of Done
- [ ] Multiple crime types with distinct effects
- [ ] Crime events generated from conditions
- [ ] Police response and arrest mechanics
- [ ] Prison capacity tracking
- [ ] Crime overlay shows types and rates

## Test Plan
- Unit: High poverty + low police = high crime rate
- Integration: Build police station, verify crime decreases in coverage area
- Integration: Full prison results in released criminals and higher recidivism

## Pitfalls
- crime.rs already exists with basic implementation -- extend carefully
- Crime death spiral: crime -> low land value -> poverty -> more crime
- Need policy tools to break the cycle (community programs, job training)

## Relevant Code
- `crates/simulation/src/crime.rs` -- expand crime system
- `crates/simulation/src/services.rs:ServiceType::PoliceStation` -- police capacity
- `crates/simulation/src/happiness.rs` -- crime happiness penalty
