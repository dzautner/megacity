# MILE-002: Unique Buildings and Monument Unlocks
**Priority:** T3
**Complexity:** M
**Dependencies:** MILE-001
**Source:** cities_skylines_analysis.md, section 13.2

## Description
Implement unique buildings unlocked by meeting specific gameplay conditions (not just population). These are aspirational goals that reward specific achievements and provide large bonuses.

- Condition-based unlocks (10 minimum):
  - "City Hospital": cure 1000 sick citizens
  - "Grand Mall": have commercial buildings reach level 3 in 5+ districts
  - "Tech Hub": have 5000 university-educated workers
  - "Green Paradise": zero pollution for 6 months
  - "Transit Master": 10,000 transit riders/day
  - "Crime Fighter": reduce crime to < 5% city-wide
  - "Heritage City": 5+ historic districts
  - "Economic Powerhouse": $1M monthly income
  - "Vertical City": 20+ level-5 buildings
  - "Tourist Mecca": 5000 tourists per month

- Each unique building: large land value boost, tourism attraction, unique mesh
- Monuments: require all unique buildings in category

## Definition of Done
- [ ] 10+ unique buildings with condition unlocks
- [ ] Conditions checked automatically
- [ ] Unique buildings provide significant bonuses
- [ ] Monument system requires multiple uniques

## Test Plan
- Integration: Meet "Tech Hub" condition, verify unlock notification
- Integration: Place unique building, verify land value and tourism boost

## Pitfalls
- achievements.rs already has partial implementation
- Conditions must be trackable with existing data (no new metrics needed per unique)
- Unique building meshes need distinct visual designs

## Relevant Code
- `crates/simulation/src/achievements.rs` -- condition tracking
- `crates/simulation/src/unlocks.rs` -- unlock gating
- `crates/rendering/src/building_meshes.rs` -- unique building meshes
