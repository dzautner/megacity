# SVC-011: Emergency Management System

**Priority:** T3 (Differentiation)
**Complexity:** Medium (2 person-weeks)
**Dependencies:** SVC-003 (vehicle dispatch)
**Source:** historical_demographics_services.md Section 5.5

## Description

Emergency Operations Center (EOC) building provides disaster coordination. Having EOC reduces disaster severity by 30%. Emergency sirens (automatic warning) reduce casualty rate by 20%. Emergency shelters (designated buildings) provide evacuation housing. Without EOC, disaster response is uncoordinated: +50% response time, +100% casualties. EOC costs ~$500K-2M/year. Invisible service until disaster strikes, rewarding proactive investment.

## Definition of Done

- [ ] EOC service building type
- [ ] Emergency siren building type (coverage radius)
- [ ] Emergency shelter designation for existing buildings
- [ ] EOC reduces disaster severity by 30%
- [ ] Sirens reduce casualty rate by 20%
- [ ] Without EOC: response time +50%, casualties +100%
- [ ] Shelter capacity for evacuation
- [ ] Disaster preparedness metric displayed in stats

## Test Plan

- Unit test: EOC presence reduces disaster damage by 30%
- Unit test: siren coverage reduces casualties by 20%
- Integration test: prepared city survives disaster with less damage

## Pitfalls

- Player may not build EOC until after first disaster; design should make this a learning moment
- EOC interaction with existing disaster system in disasters.rs

## Relevant Code

- `crates/simulation/src/disasters.rs` (ActiveDisaster, apply_earthquake_damage)
- `crates/simulation/src/services.rs` (ServiceType enum)
