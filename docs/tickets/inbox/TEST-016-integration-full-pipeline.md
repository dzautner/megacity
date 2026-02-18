# TEST-016: Integration Test: Full City Growth Pipeline

## Priority: T1 (Core)
## Effort: Medium (2-3 days)
## Source: testing_strategy.md -- Section 3.3: Full Causal Chain

## Description
Ultimate integration test: empty world -> place cross-shaped roads -> zone R and C -> run 500 ticks -> verify buildings, citizens, economy, traffic all functioning.

## Acceptance Criteria
- [ ] Cross-shaped road network placed
- [ ] Residential on one quadrant, commercial on another
- [ ] After 500 ticks: buildings.len() > 0
- [ ] citizens.len() > 0
- [ ] budget.monthly_income > 0 or last_collection_day > 0
- [ ] CityStats.population matches citizen count
