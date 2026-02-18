# INFRA-103: Disaster Preparedness and Terrain-Aware Disasters
**Priority:** T4
**Complexity:** M (1-3 days)
**Dependencies:** INFRA-015, INFRA-014
**Source:** master_architecture.md, M5

## Description
Enhance disaster system with preparedness mechanics and terrain awareness. Earthquakes stronger near fault lines (INFRA-015). Floods worse in flood plains (INFRA-014). Tornado damage follows terrain corridors. Preparedness buildings: seismically reinforced structures (reduce earthquake damage 50%), flood barriers (reduce flood damage), early warning systems (citizen evacuation before disaster). Evacuation routes along road network.

## Definition of Done
- [ ] Disaster damage scales with terrain features
- [ ] Preparedness buildings reduce specific disaster damage
- [ ] Early warning system allows evacuation time
- [ ] Evacuation route computation on road network
- [ ] Preparedness score displayed per disaster type
- [ ] Tests pass

## Test Plan
- Unit: Earthquake damage 2x near fault lines vs far
- Unit: Flood barrier reduces flood damage by expected percentage
- Integration: Prepared city recovers faster from disaster

## Pitfalls
- Evacuation simulation is expensive; simplify to zone-based evacuation
- Preparedness buildings add another service type to manage
- Terrain-aware disasters require terrain features to exist first

## Relevant Code
- `crates/simulation/src/disasters.rs` -- disaster system
- `crates/simulation/src/terrain.rs` -- terrain features (fault lines, flood plains)
