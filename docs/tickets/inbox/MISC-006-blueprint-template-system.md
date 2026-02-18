# MISC-006: Blueprint/Template System
**Priority:** T4
**Complexity:** L
**Dependencies:** CSLN-006
**Source:** master_architecture.md, section M5

## Description
Allow players to save road/zone layouts as reusable blueprints. Select an area, save as template, stamp down copies elsewhere. Essential for repeating road patterns efficiently.

- Select rectangular area of roads + zones
- Save as named blueprint (stores cell types, zones, road types relative to origin)
- Blueprint palette: browse saved blueprints
- Stamp blueprint onto map (preview before placement)
- Cost: sum of all road/building costs in blueprint
- Share blueprints between saves

## Definition of Done
- [ ] Area selection for blueprint capture
- [ ] Blueprint save and naming
- [ ] Blueprint palette UI
- [ ] Blueprint stamping with preview
- [ ] Blueprints persist across game sessions

## Test Plan
- Integration: Save blueprint, load in new area, verify correct placement

## Pitfalls
- Blueprint placement must handle terrain conflicts
- Blueprints with buildings need to handle building spawn vs just zoning
- Blueprint file format needs to be compact and versioned

## Relevant Code
- `crates/rendering/src/input.rs` -- blueprint capture and stamp tools
- `crates/save/src/lib.rs` -- blueprint serialization
