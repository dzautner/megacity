# INFRA-088: 15-Minute City Walkability Scoring
**Priority:** T3
**Complexity:** M (1-3 days)
**Dependencies:** none
**Source:** master_architecture.md, M4

## Description
Compute walkability score per cell: can a resident reach essential services (grocery, school, healthcare, park, transit) within a 15-minute walk (~1.2km, ~75 cells)? Score = weighted count of accessible service types / total service types. Display as overlay and per-district metric. Gameplay bonus: areas scoring 80%+ get happiness bonus and land value increase.

## Definition of Done
- [ ] Walkability score computed per cell (0-100%)
- [ ] Essential service types defined (6-8 categories)
- [ ] Walking distance computed on road/path network
- [ ] Walkability overlay mode
- [ ] Per-district walkability metric in stats
- [ ] Happiness and land value bonus for high walkability
- [ ] Tests pass

## Test Plan
- Unit: Cell with all essential services within 75 cells = 100% score
- Unit: Cell with no nearby services = 0% score
- Integration: Dense mixed-use areas score higher than suburban sprawl

## Pitfalls
- Walking distance is network distance, not Euclidean
- Some services (hospital) are rare; adjust weight or distance threshold
- 15-minute walking distance varies by age/mobility

## Relevant Code
- `crates/simulation/src/services.rs` -- service locations
- `crates/simulation/src/happiness.rs` -- walkability happiness bonus
- `crates/rendering/src/overlay.rs` -- walkability overlay
