# ZONE-007: NIMBY/YIMBY Citizen Mechanics
**Priority:** T3
**Complexity:** L
**Dependencies:** ZONE-001, BLDG-003
**Source:** urban_planning_zoning.md, section 6.1; master_architecture.md, section 3

## Description
Implement citizen opposition/support mechanics for zoning changes and new development. Citizens near proposed developments react based on their personality, property values, and the type of development.

- When player rezones or places a high-impact building, nearby citizens generate opinion
- NIMBY factors: density increase, industrial adjacency, traffic increase, "different" income level
- YIMBY factors: amenity addition (parks, transit), job creation, housing need
- Citizen opposition reduces happiness and can trigger protests (visual event)
- Opposition strength scales with land value (wealthy neighborhoods oppose more)
- High opposition can slow construction or reduce building level-up speed
- Player can use policies to override (eminent domain) at happiness cost

## Definition of Done
- [ ] Rezoning triggers citizen opinion calculation in affected radius
- [ ] Opposition generates happiness penalty for nearby residents
- [ ] High opposition visible as protest events
- [ ] Opposition affects construction speed/probability
- [ ] Player can override via policy at cost

## Test Plan
- Unit: High-density rezone near wealthy low-density generates strong opposition
- Unit: Park placement generates support (YIMBY)
- Integration: Rezone from ResidentialLow to Industrial near houses, verify happiness drops

## Pitfalls
- Must not make zoning changes feel impossible -- opposition should be a cost, not a blocker
- Need radius of effect (how far does opposition reach? 5-10 cells)
- Citizen personality (from CitizenDetails) should influence NIMBY tendency

## Relevant Code
- `crates/simulation/src/citizen.rs:Personality` -- ambition, sociability affect NIMBY tendency
- `crates/simulation/src/happiness.rs` -- apply opposition penalty
- `crates/simulation/src/events.rs` -- create protest event
- `crates/rendering/src/input.rs` -- trigger on rezone action
