# ZONE-013: 15-Minute City Walkability Scoring
**Priority:** T3
**Complexity:** M
**Dependencies:** none
**Source:** urban_planning_zoning.md, section 5.1-5.2; master_architecture.md, section 3

## Description
Implement a per-cell walkability score based on the 15-minute city concept. Each cell gets a score (0-100) based on how many essential services and amenities are reachable within a 15-minute walk (~1200m = 75 grid cells at 16m/cell).

- Categories scored: grocery/commercial, school, healthcare, park, transit, employment
- Each category 0-100 based on distance to nearest qualifying building
- Walk Score methodology: full points within 400m, decay to 0 at 1600m
- Composite score = weighted average of category scores
- Display as overlay (green = highly walkable, red = car-dependent)
- Score affects happiness, land value, and citizen mode choice

## Definition of Done
- [ ] Per-cell walkability score calculated
- [ ] Score reflects distance to essential services
- [ ] Walkability overlay implemented
- [ ] Score feeds into happiness and land value calculations
- [ ] Score affects citizen transit/walk mode choice

## Test Plan
- Unit: Cell adjacent to commercial, school, park, and transit scores > 80
- Unit: Cell with no services within 1600m scores < 20
- Integration: Verify overlay correctly shows walkable vs car-dependent areas

## Pitfalls
- Computing walkability for all 65K cells every tick is expensive -- use slow tick timer
- Distance should ideally be network distance (road/path), not Euclidean
- Weighting of categories needs tuning to avoid one category dominating

## Relevant Code
- `crates/simulation/src/land_value.rs` -- integrate walkability into land value
- `crates/simulation/src/happiness.rs` -- walkability happiness component
- `crates/rendering/src/overlay.rs` -- add walkability overlay
- `crates/simulation/src/services.rs` -- service building locations for scoring
