# BLDG-012: Building Abandonment Lifecycle
**Priority:** T1
**Complexity:** M
**Dependencies:** none
**Source:** urban_planning_zoning.md, section 3.5; master_architecture.md, section 5.1

## Description
Implement a multi-stage abandonment lifecycle: Occupied -> Declining -> Vacant -> Abandoned -> Auto-demolished. Currently abandonment.rs exists but needs tuning and the full degradation visual pipeline.

- Declining: occupancy dropping for 6-12 game-months + poor services. Building stays but slowly loses occupants.
- Vacant: occupancy hits 0. Building remains for 3-6 game-months. Visual: some windows dark.
- Abandoned: visual degradation (boarded windows, graffiti). Crime +5 in radius. 12 game-months.
- Auto-demolish: building entity despawned, cell cleared, rubble visual briefly.
- Player can intervene at any stage by improving services
- Abandoned buildings spread blight (reduce land value in radius)

## Definition of Done
- [ ] Multi-stage lifecycle: declining, vacant, abandoned, demolished
- [ ] Each stage has distinct visual
- [ ] Crime increase from abandoned buildings
- [ ] Land value reduction from abandoned buildings
- [ ] Player can reverse decline by improving conditions
- [ ] Auto-demolish after extended abandonment

## Test Plan
- Unit: Building with 0 occupants and no demand transitions to declining
- Integration: Remove services from area, verify buildings progress through abandonment stages
- Integration: Add services to declining area, verify recovery

## Pitfalls
- Must handle citizens being evicted gracefully (need new homes/workplaces)
- Auto-demolish must properly clean up all entity references
- Blight radius from abandoned buildings can create death spirals

## Relevant Code
- `crates/simulation/src/abandonment.rs` -- expand existing system
- `crates/simulation/src/buildings.rs:Building` -- occupancy tracking
- `crates/simulation/src/crime.rs` -- abandoned building crime modifier
- `crates/simulation/src/land_value.rs` -- blight radius effect
