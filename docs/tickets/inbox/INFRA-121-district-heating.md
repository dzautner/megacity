# INFRA-121: District Heating/Cooling Network
**Priority:** T3
**Complexity:** L (3-5 days)
**Dependencies:** INFRA-019
**Source:** infrastructure_engineering.md, Section 9

## Description
Implement district heating/cooling as underground pipe network. Central CHP plant produces electricity AND heat (60-80% efficiency vs 33-45% for electricity-only). Hot water pipes distribute heat to buildings. Reduces building-level heating costs. Only economical in dense areas (pipe network cost per building decreases with density). Integrates with `heating.rs` system. Optional ground-source heat pump district systems.

## Definition of Done
- [ ] District heating network (underground pipe layer)
- [ ] CHP plant producing both power and heat
- [ ] Connected buildings get heating at reduced cost
- [ ] Density threshold for economic viability
- [ ] Integration with existing heating system
- [ ] Tests pass

## Test Plan
- Unit: CHP plant efficiency 70% vs standalone power plant 40%
- Unit: Connected buildings save 30-40% on heating costs
- Integration: Dense downtown district benefits from heating network

## Pitfalls
- District heating only economic above minimum density (~50 buildings/km)
- CHP plant must be near the district (heat loss over distance)
- Existing `heating.rs` handles per-building heating; integrate or replace

## Relevant Code
- `crates/simulation/src/heating.rs` -- existing heating system
- `crates/simulation/src/utilities.rs` -- pipe network patterns
