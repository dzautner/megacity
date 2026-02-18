# INFRA-095: Blueprint/Template System
**Priority:** T4
**Complexity:** M (1-3 days)
**Dependencies:** none
**Source:** master_architecture.md, M5

## Description
Allow players to save and reuse layouts as blueprints. Select a rectangular area, save road layout + zoning as a template. Place the blueprint elsewhere on the map. Blueprints include road types, zone types, and service building positions. Built-in blueprints for common patterns: grid block, cul-de-sac, roundabout intersection, transit-oriented development.

## Definition of Done
- [ ] Area selection tool for blueprint capture
- [ ] Blueprint saved as reusable template
- [ ] Blueprint placement with preview
- [ ] Built-in blueprint library (5+ common patterns)
- [ ] Custom blueprints saved to file
- [ ] Tests pass

## Test Plan
- Unit: Captured blueprint matches original layout
- Unit: Placed blueprint creates correct roads and zones
- Integration: Player creates custom blueprint and reuses it multiple times

## Pitfalls
- Blueprint placement must handle terrain conflicts (water, slope)
- Road connections at blueprint edges need to snap to existing roads
- Blueprint rotation/mirroring adds complexity but is very useful

## Relevant Code
- `crates/rendering/src/input.rs` -- selection and placement tools
- `crates/simulation/src/grid.rs` -- grid data capture/restore
