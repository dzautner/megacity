# BLDG-011: Organic Densification (Low to High Density Transition)
**Priority:** T3
**Complexity:** M
**Dependencies:** BLDG-005, ZONE-006
**Source:** urban_planning_zoning.md, section 3.6

## Description
Allow ResidentialLow zones to organically densify into ResidentialHigh when market conditions support it (high land value, high demand, neighbors already dense). This mimics real-world gentle density: house -> duplex -> 4-plex -> mid-rise.

- should_densify() checks: land_value > threshold, demand > 0.5, building at max level for current zone, avg neighbor levels high
- Densification probability increases with sustained high land value
- When densifying: demolish current building, change cell zone to higher density, allow higher-level building to spawn
- Densification threshold tuned per zone: ResidentialLow needs LV > 120, CommercialLow needs LV > 100
- Only densifies one step at a time (Low -> Medium, Medium -> High)
- Requires ZONE-001 (ResidentialMedium) as intermediate step

## Definition of Done
- [ ] Buildings at max level with high land value + demand can densify
- [ ] Cell zone changes from Low to Medium (or Medium to High)
- [ ] Densification is gradual and organic-looking
- [ ] Visual transition visible (building demolished, new denser building appears)

## Test Plan
- Unit: should_densify returns true when conditions met
- Unit: should_densify returns false when land_value < threshold
- Integration: Sustained high land value in low-density area triggers densification

## Pitfalls
- Must evict residents before demolition
- Zone change affects zone demand accounting (cell moves from R_low to R_med)
- Player may not want densification -- add policy to lock density (high-rise ban equivalent)

## Relevant Code
- `crates/simulation/src/buildings.rs` -- densification check system
- `crates/simulation/src/grid.rs:Cell` -- zone type change
- `crates/simulation/src/zones.rs` -- demand accounting
