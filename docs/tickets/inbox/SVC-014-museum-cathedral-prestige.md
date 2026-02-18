# SVC-014: Cultural Buildings (Museum, Cathedral, Stadium) Prestige

**Priority:** T3 (Differentiation)
**Complexity:** Low (0.5 person-weeks)
**Dependencies:** None
**Source:** historical_demographics_services.md Section 5, master_architecture.md

## Description

Cultural buildings provide city-wide prestige bonus and tourism attraction. Museum: +5 happiness in radius, +10 tourism attraction, +5 education quality. Cathedral: +5 happiness, +8 tourism, community building for religious citizens. Stadium: +3 happiness city-wide during events, +15 tourism during events, large land use. TVStation: +5 city-wide visibility (immigration boost), entertainment coverage. Each has unique effects beyond coverage radius.

## Definition of Done

- [ ] Museum: happiness, tourism, education bonuses
- [ ] Cathedral: happiness, tourism, community bonuses
- [ ] Stadium: periodic event bonuses (weekly/monthly)
- [ ] TVStation: immigration and entertainment bonuses
- [ ] Prestige metric from cultural building count
- [ ] Tourism attraction from cultural buildings
- [ ] City-wide effects (not just radius)

## Test Plan

- Unit test: museum provides education quality bonus
- Unit test: stadium event provides temporary happiness boost
- Integration test: cultural buildings attract tourists

## Pitfalls

- Stadium events should be periodic, not constant bonus
- Buildings exist in ServiceType but may lack functional differentiation

## Relevant Code

- `crates/simulation/src/services.rs` (Museum, Cathedral, Stadium, TVStation)
- `crates/simulation/src/tourism.rs`
