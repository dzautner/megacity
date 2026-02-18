# TEST-018: Integration Test: Fire Spread and Extinguish

## Priority: T2 (Depth)
## Effort: Small (1-2 days)
## Source: testing_strategy.md -- Section 3.5: Emergency Scenarios

## Description
Place buildings and fire station. Manually start fire. Run ticks. Verify fire spreads but is eventually contained if fire station coverage is adequate.

## Acceptance Criteria
- [ ] Fire grid manually set at (50, 50)
- [ ] Buildings placed in fire spread range
- [ ] Fire station placed with coverage
- [ ] After 200 ticks, fire is contained
- [ ] Without fire station, fire spreads further
