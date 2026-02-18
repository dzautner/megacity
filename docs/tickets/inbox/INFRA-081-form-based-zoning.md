# INFRA-081: Form-Based Zoning System
**Priority:** T3
**Complexity:** L (3-5 days)
**Dependencies:** INFRA-055
**Source:** master_architecture.md, M4

## Description
Implement form-based zoning as an alternative to Euclidean (use-based) zoning. Form-based zoning controls building form (height, setback, frontage type) rather than use. Zones: T1 Rural, T2 Suburban, T3 Sub-Urban, T4 General Urban, T5 Urban Center, T6 Urban Core. Higher T = taller buildings, no setback, mixed use. Allows mixed-use buildings (commercial ground floor + residential above) naturally. Player chooses between Euclidean and form-based zoning systems.

## Definition of Done
- [ ] `FormBasedZone` enum: T1-T6
- [ ] Form controls: max height, setback, lot coverage
- [ ] Mixed-use allowed in T4-T6
- [ ] Visual difference between form-based and Euclidean neighborhoods
- [ ] Player toggle between zoning systems
- [ ] Tests pass

## Test Plan
- Unit: T6 zone allows 10+ story buildings; T2 allows 2 stories max
- Unit: Mixed-use building spawns in T4+ zone
- Integration: Form-based district looks visually distinct from Euclidean

## Pitfalls
- Two zoning systems adds complexity; may confuse new players
- Mixed-use buildings need new mesh types (shops on ground floor)
- Transitioning from one system to another mid-game is complex

## Relevant Code
- `crates/simulation/src/zones.rs` -- zone types
- `crates/simulation/src/buildings.rs` -- building spawn rules
