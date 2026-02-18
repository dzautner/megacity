# TEST-007: Unit Tests for Service Coverage Grid

## Priority: T1 (Core)
## Effort: Small (1 day)
## Source: testing_strategy.md -- Section 2: Unit Testing

## Description
Test service coverage BFS/radius calculation. Verify cells within radius have coverage, cells outside do not. Test multiple overlapping service buildings.

## Acceptance Criteria
- [ ] Test single service building covers correct radius
- [ ] Test cells just outside radius have no coverage
- [ ] Test overlapping coverage from multiple buildings
- [ ] Test all service types (fire, police, health, education, garbage)
- [ ] Test coverage bitflags are correctly set
