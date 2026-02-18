# CIT-066: NIMBY/YIMBY Citizen Mechanic

**Priority:** T3 (Differentiation)
**Complexity:** Medium (3-4 person-weeks)
**Dependencies:** GOV-001 (factions)
**Source:** master_architecture.md Section 1.3

## Description

Citizens oppose or support nearby development changes. NIMBY: homeowners near proposed high-density/industrial rezoning oppose change. YIMBY: renters and young adults support density. Opposition probability = homeowner * (property_value_risk * 0.5 + change_magnitude * 0.3 + personality * 0.2). When opposition > 50% of affected residents, protest event or referendum triggered. Overriding opposition costs civic trust. Supporting popular development gains trust.

## Definition of Done

- [ ] Opposition calculation for rezoning near existing residents
- [ ] Property value risk assessment (will this lower my home value?)
- [ ] Homeowner vs renter opposition difference
- [ ] Opposition threshold for protest/referendum trigger
- [ ] Override mechanic with civic trust cost
- [ ] NIMBY faction alignment (GOV-001)
- [ ] Visual: protest markers near opposed development

## Test Plan

- Unit test: homeowner near proposed factory = high opposition
- Unit test: renter near proposed transit = low opposition
- Unit test: >50% opposition triggers event
- Integration test: rezoning wealthy neighborhood generates NIMBY protest

## Pitfalls

- NIMBY shouldn't block all development; player must have override option
- Must balance realism with playability

## Relevant Code

- `crates/simulation/src/zones.rs`
- `crates/simulation/src/events.rs`
- GOV-001 infrastructure
