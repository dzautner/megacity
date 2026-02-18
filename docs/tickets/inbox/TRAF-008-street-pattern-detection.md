# TRAF-008: Street Pattern Detection and Scoring
**Priority:** T3
**Complexity:** M
**Dependencies:** none
**Source:** urban_planning_zoning.md, section 4.7

## Description
Implement automatic detection of street patterns (grid, radial, organic, cul-de-sac, superblock) and provide gameplay bonuses/penalties based on pattern type.

- Detect patterns from road network topology: intersection density, dead-end ratio, connectivity index, block size variance
- Grid pattern: high regularity, high connectivity (bonus: route efficiency)
- Radial: spoke count >= 6, ring count >= 2 (bonus: monumental character)
- Organic: high angle/size variance, narrow streets (bonus: heritage/tourism)
- Cul-de-sac: high dead-end ratio (penalty: traffic on collectors, low walkability)
- Superblock: grid with interior pedestrian conversion (bonus: happiness, land value)
- Pattern analysis displayed in city statistics panel

## Definition of Done
- [ ] Street pattern detected per district
- [ ] Pattern type identified with confidence score
- [ ] Bonuses/penalties applied based on pattern
- [ ] Pattern type shown in district info

## Test Plan
- Integration: Build grid pattern, verify Grid detection
- Integration: Build cul-de-sac suburbs, verify penalty applied

## Pitfalls
- Mixed patterns common (residential cul-de-sac + commercial grid)
- Detection algorithms need tuning thresholds
- Per-district analysis, not city-wide (different areas have different patterns)

## Relevant Code
- `crates/simulation/src/districts.rs` -- pattern analysis per district
- `crates/simulation/src/traffic.rs` -- connectivity metrics
- `crates/simulation/src/happiness.rs` -- pattern-based bonuses
