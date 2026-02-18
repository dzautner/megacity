# INFRA-027: Metro Station Placement and Design
**Priority:** T2
**Complexity:** L (3-5 days)
**Dependencies:** INFRA-026
**Source:** underground_infrastructure.md, Metro Station section

## Description
Implement metro station placement along tunnel routes. Station types: side platform (basic, $50K), island platform ($60K), stacked transfer ($120K), cross-platform transfer ($150K). Each station occupies 3-5 cells along tunnel. Stations have entrance/exit cells on surface (must connect to road). Platform length determines max train length. Station catchment radius (800m/~50 cells) for ridership calculation. Stations interact with land value (increase nearby values).

## Definition of Done
- [ ] `MetroStation` struct with type, position, platform length, entrances
- [ ] Station placement snaps to tunnel route
- [ ] Surface entrances must connect to road network
- [ ] Station catchment radius for ridership
- [ ] Land value bonus from metro station proximity
- [ ] Station cost by type
- [ ] Tests pass

## Test Plan
- Unit: Station placed on tunnel is valid; station off-tunnel rejected
- Unit: Land value increases within catchment radius

## Pitfalls
- Transfer stations where two lines cross need special handling
- Surface entrance placement must avoid existing buildings
- Station at edge of map may have truncated catchment

## Relevant Code
- `crates/simulation/src/land_value.rs` -- land value bonus
- `crates/simulation/src/services.rs` -- service coverage pattern
