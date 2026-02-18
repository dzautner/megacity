# REND-003: Building Mesh Variants (2-3 per Zone/Level)
**Priority:** T1
**Complexity:** M
**Dependencies:** none
**Source:** master_architecture.md, section M2

## Description
Create at least 2-3 distinct building mesh variants per zone type per level. Currently one mesh per zone/level creates monotonous cityscapes. This is the minimum for M2 visual quality.

- ResidentialLow L1: small house, cottage, ranch house
- ResidentialLow L2: larger house, duplex, row house
- ResidentialLow L3: small apartment, townhouse complex
- ResidentialHigh L1: apartment block, mid-rise
- ResidentialHigh L3: tall apartment tower, luxury tower
- CommercialLow L1: corner store, cafe, small shop
- CommercialHigh L1: strip mall, retail store
- Industrial L1: warehouse, small factory
- Office L1: small office, professional building
- Selection: hash(position + seed) to choose variant (deterministic)

## Definition of Done
- [ ] At least 2 mesh variants per zone/level combo for all 6 zone types
- [ ] Variant selection deterministic (same city looks the same each time)
- [ ] Visual distinction between variants clear
- [ ] Variants appropriate for zone type and level

## Test Plan
- Visual: City has visible building variety within same zone
- Unit: Same position always selects same variant

## Pitfalls
- Procedural mesh generation in building_meshes.rs must create distinct shapes
- More variants = more GPU memory (but these are small meshes)
- Must maintain consistent aesthetic within zone type

## Relevant Code
- `crates/rendering/src/building_meshes.rs` -- add variant generators
- `crates/rendering/src/building_render.rs` -- select variant by hash
