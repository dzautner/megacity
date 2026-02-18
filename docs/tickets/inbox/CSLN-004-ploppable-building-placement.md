# CSLN-004: Ploppable Building Placement (RICO System)
**Priority:** T3
**Complexity:** M
**Dependencies:** BLDG-003
**Source:** cities_skylines_analysis.md, section 17.3

## Description
CS1's Ploppable RICO mod had ~3M subscribers. Players want to manually place specific buildings instead of relying on zone auto-growth RNG. Implement manual building placement as an alternative to zone-based growth.

- Player can select from building catalog and place specific buildings on zoned cells
- Placed buildings skip the random pool selection (exact model chosen by player)
- Placed buildings still need correct zone type and road adjacency
- Construction cost deducted from treasury
- Placed buildings can be any level (but require services to maintain level)
- Toggle: allow/disallow player-placed buildings in districts

## Definition of Done
- [ ] Building catalog UI showing available buildings per zone type
- [ ] Player can select and place specific building on valid cell
- [ ] Construction cost deducted
- [ ] Building functions normally after placement
- [ ] Works alongside automatic zone growth

## Test Plan
- Integration: Place specific building, verify it appears and functions
- Integration: Place high-level building without services, verify it downgrades

## Pitfalls
- Must not bypass zone requirements (still need correct zone type)
- Building catalog needs UI work (thumbnails, stats, filtering)
- Must not make auto-growth obsolete (balance placement cost)

## Relevant Code
- `crates/rendering/src/input.rs` -- building placement tool
- `crates/simulation/src/buildings.rs` -- manual spawn function
- `crates/ui/src/toolbar.rs` -- building catalog panel
