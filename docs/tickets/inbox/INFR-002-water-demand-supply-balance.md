# INFR-002: Water Demand/Supply Balance with Shortage
**Priority:** T1
**Complexity:** M
**Dependencies:** none
**Source:** master_architecture.md, section M2; cities_skylines_analysis.md, section 10

## Description
Implement water demand/supply balance. Like power, water currently propagates without capacity limits. Buildings should consume water based on zone/level. When supply < demand, water shortages cause health problems and fire risk.

- Water demand per building: scales with population/workers
- Water supply: from water pumps (capacity in m3/day)
- Shortage effects: health problems (citizens get sick), fire risk increases (no water for hydrants), buildings cannot upgrade, happiness -10
- Sewage: generated at same rate as water consumption, must be treated
- Untreated sewage causes water pollution downstream

## Definition of Done
- [ ] Water demand per building calculated
- [ ] Water supply from pumps/treatment tracked
- [ ] Shortage triggers health and fire penalties
- [ ] Water supply/demand in utility panel
- [ ] Sewage tracking and treatment

## Test Plan
- Integration: Remove water pump, verify shortage effects
- Integration: Verify sewage generation matches water consumption

## Pitfalls
- Current BFS propagation has no capacity tracking
- Water pump placement near polluted water creates contamination
- Sewage output point must not be upstream of water intake

## Relevant Code
- `crates/simulation/src/utilities.rs` -- water propagation
- `crates/simulation/src/health.rs` -- water shortage health effects
- `crates/simulation/src/fire.rs` -- no-water fire risk increase
