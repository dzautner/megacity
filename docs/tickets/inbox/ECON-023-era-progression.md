# ECON-023: Era Progression System
**Priority:** T3
**Complexity:** L
**Dependencies:** BLDG-003
**Source:** urban_planning_zoning.md, section 3.4; master_architecture.md, section M4

## Description
Implement building era progression where the visual style of new buildings changes as the city ages. Older buildings retain their era, creating visual neighborhood diversity reflecting real-world architectural history.

- Eras: PreWar (before year 40), MidCentury (40-70), Late20th (70-100), Modern (100+)
- Game starts in PreWar era
- New buildings use current era's architectural style
- Existing buildings retain their construction era
- Era affects: BuildingAppearance materials, colors, roof styles
- Historic districts preserve old-era buildings (ZONE-008 integration)
- Era transition event notification

## Definition of Done
- [ ] BuildingEra enum with 4 eras
- [ ] Current era determined by game clock
- [ ] New buildings spawn with current era style
- [ ] Old buildings retain original era
- [ ] Visual distinction between eras

## Test Plan
- Integration: Play 50 game-years, verify new buildings look different from originals

## Pitfalls
- Need distinct visual styles per era (art/mesh work)
- Game time compression: how many real-time hours = one era?
- PreWar buildings in Modern city should look charmingly old, not buggy

## Relevant Code
- `crates/simulation/src/buildings.rs` -- era assignment at construction
- `crates/rendering/src/building_meshes.rs` -- era-specific mesh generation
- `crates/simulation/src/time_of_day.rs:GameClock` -- year tracking
