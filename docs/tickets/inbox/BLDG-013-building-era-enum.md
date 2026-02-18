# BLDG-013: BuildingEra Enum and Per-Building Style
**Priority:** T2
**Complexity:** S
**Dependencies:** BLDG-003
**Source:** urban_planning_zoning.md, section 3.4

## Description
Add BuildingEra enum (PreWar, MidCentury, Late20th, Modern) to building data. Each building stores its construction era, which determines visual style, material palette, and maintenance characteristics.

- PreWar: ornate, brick, detailed. Higher maintenance but higher heritage value
- MidCentury: simple, functional, boxy. Low maintenance, efficient
- Late20th: postmodern, varied materials. Moderate maintenance
- Modern: glass, steel, minimalist. Low maintenance, high efficiency
- Era stored on Building component at construction time
- Era affects BuildingAppearance material and color ranges

## Definition of Done
- [ ] BuildingEra enum defined
- [ ] Era assigned to Building at construction
- [ ] Era affects visual appearance
- [ ] Era serialized in save file

## Test Plan
- Unit: Building constructed in game-year 30 gets PreWar era
- Unit: Building constructed in game-year 80 gets Late20th era

## Pitfalls
- Must not change era of existing buildings when game clock advances
- Era boundaries need tuning with game-time scale

## Relevant Code
- `crates/simulation/src/buildings.rs:Building` -- add era field
- `crates/rendering/src/building_meshes.rs` -- era-specific generation
