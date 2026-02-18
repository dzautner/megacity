# CRIME-003: Policing Strategies (Preventive vs Reactive)

**Priority:** T2 (Depth)
**Complexity:** Medium (2 person-weeks)
**Dependencies:** CRIME-001 (crime types)
**Source:** social_agent_simulation.md Section 8.3, historical_demographics_services.md Section 3.2

## Description

Police effectiveness depends on strategy choice (policy setting). Preventive (community policing): -30% crime base but costs 1.5x budget, builds trust slowly. Reactive (response-focused): faster response time but no prevention bonus. Aggressive (zero tolerance): -40% crime short-term but trust erodes, protests at trust < 20%. Trust level affects witness cooperation, crime reporting rate. Police staffing ratio: target 2.4 officers per 1000 pop (US average). Under/over-staffing affects effectiveness.

## Definition of Done

- [ ] `PolicingStrategy` enum (Preventive, Reactive, Aggressive) in Policies
- [ ] Trust level resource (0-100, starts at 50)
- [ ] Preventive: -30% crime, 1.5x cost, +2 trust/month
- [ ] Reactive: baseline crime, 1.0x cost, neutral trust
- [ ] Aggressive: -40% crime, 1.2x cost, -3 trust/month
- [ ] Trust below 20 triggers protest events
- [ ] Police staffing ratio metric (officers per 1000 pop)
- [ ] Under-staffing reduces effectiveness proportionally

## Test Plan

- Unit test: preventive strategy reduces crime by 30%
- Unit test: aggressive strategy with trust < 20 triggers protest
- Unit test: under-staffed force has reduced effectiveness
- Integration test: switching strategies shows gradual effect over months

## Pitfalls

- Trust is a slow-moving variable; instant strategy changes shouldn't cause instant trust changes
- Police budget already exists in ExtendedBudget; must integrate not duplicate

## Relevant Code

- `crates/simulation/src/crime.rs` (update_crime)
- `crates/simulation/src/policies.rs` (Policies)
- `crates/simulation/src/services.rs` (PoliceStation, PoliceKiosk, PoliceHQ)
