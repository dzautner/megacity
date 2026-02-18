# WATER-006: Storm Drain and Retention Pond Infrastructure

## Priority: T2 (Depth)

## Description
Implement storm drainage infrastructure that removes stormwater before it causes flooding. Storm drains follow roads, retention ponds store excess water, and green infrastructure (rain gardens, bioswales) provides natural drainage.

## Current State
- No storm drain system exists.
- No retention/detention pond building.
- No green infrastructure for stormwater.

## Definition of Done
- [ ] Storm drains: auto-follow road placement, each drain removes 0.5 in/hr capacity.
- [ ] Retention pond: 4x4 building, stores 500,000 gallons, slowly releases.
- [ ] Rain garden: 1x1 building, absorbs 100% of local cell runoff + 50% from 4 neighbors.
- [ ] Bioswale: 1x2 along roads, filters and slows runoff.
- [ ] Drainage network capacity: sum of all drains in a watershed area.
- [ ] When runoff exceeds drainage capacity, flooding begins (WATER-005).
- [ ] Maintenance cost for each drainage type.

## Test Plan
- [ ] Unit test: storm drain at road removes 0.5 in/hr from local runoff.
- [ ] Unit test: retention pond fills and slowly releases.
- [ ] Integration test: city with full storm drain coverage has no flooding at moderate rain.
- [ ] Integration test: insufficient drainage causes localized flooding.

## Pitfalls
- Storm drains automatically placing along roads adds complexity to road placement.
- Retention pond sizing must match realistic water volumes.
- Green infrastructure is a separate building type that needs art assets.

## Code References
- `crates/simulation/src/grid.rs`: `CellType::Road`
- `crates/simulation/src/services.rs`: new service types needed
- Research: `environment_climate.md` sections 2.3.3, 2.4.3
