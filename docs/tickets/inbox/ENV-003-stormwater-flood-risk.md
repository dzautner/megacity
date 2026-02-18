# ENV-003: Stormwater Management and Flood Risk
**Priority:** T3
**Complexity:** L
**Dependencies:** REND-001, ENV-002
**Source:** master_architecture.md, section M4

## Description
Implement stormwater management where impervious surfaces (roads, buildings) increase runoff. Heavy rain overwhelms drainage, causing flooding in low-elevation areas. Green infrastructure (permeable pavement, rain gardens, green roofs) mitigates.

- Impervious surface ratio per cell: roads=1.0, buildings=0.9, parks=0.1, grass=0.0
- Runoff = rainfall * impervious_ratio * slope_factor
- Runoff flows downhill (terrain elevation)
- Accumulation in low points = flood risk
- Flooding: building damage, traffic blocked, health risk
- Mitigation: storm drains, retention ponds, permeable pavement, green roofs

## Definition of Done
- [ ] Impervious surface ratio tracked
- [ ] Runoff computed during rain events
- [ ] Flooding at low elevation points
- [ ] Flood damage to buildings and traffic
- [ ] Green infrastructure mitigates flooding

## Test Plan
- Integration: Heavy rain in low-elevation dense area, verify flooding
- Integration: Add retention pond, verify flood risk reduced

## Pitfalls
- Requires terrain elevation (REND-001 dependency)
- Water flow simulation can be expensive -- simplify to grid-based accumulation
- groundwater.rs already handles some water mechanics

## Relevant Code
- `crates/simulation/src/groundwater.rs` -- water accumulation
- `crates/simulation/src/weather.rs` -- rainfall data
- `crates/simulation/src/grid.rs:Cell::elevation` -- flow direction
