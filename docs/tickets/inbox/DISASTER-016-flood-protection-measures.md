# DISASTER-016: Flood Mitigation Infrastructure Suite

## Priority: T2 (Depth)

## Description
Implement the full suite of flood mitigation measures: drainage improvements, retention basins, channel improvements, elevation requirements, flood insurance, and emergency response infrastructure.

## Current State
- No flood mitigation beyond the terrain-based elevation system.

## Definition of Done
- [ ] Drainage improvements: upgradeable storm drains (-30% urban flood risk, $500K/district).
- [ ] Retention/detention basins: 4x4 buildings that store flood water temporarily.
- [ ] Channel improvements: widen/deepen river channels (+50% flood capacity, $200K/cell).
- [ ] Elevation requirements: policy requiring new buildings raised 3 ft in flood zones.
- [ ] Flood insurance program: 2% property value/year, covers 80% of flood damage.
- [ ] Emergency supplies/shelters: pre-positioned for faster recovery.
- [ ] Pump stations: active drainage during floods ($100K each).
- [ ] Each measure reduces flood damage or speeds recovery.

## Test Plan
- [ ] Unit test: detention basin stores expected water volume.
- [ ] Unit test: elevation requirement reduces building damage by 50%.
- [ ] Integration test: city with flood infrastructure has less damage from river flood.

## Pitfalls
- Many overlapping mitigation types; need clear player guidance.
- Insurance program requires financial tracking integration.
- Channel improvements change terrain near rivers.

## Code References
- `crates/simulation/src/disasters.rs`: flood processing
- Research: `environment_climate.md` sections 5.2.2, 5.2.3
