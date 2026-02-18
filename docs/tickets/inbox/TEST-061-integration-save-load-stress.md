# TEST-061: Stress Test: Rapid Save/Load Cycles

## Priority: T2 (Depth)
## Effort: Small (1 day)
## Source: testing_strategy.md -- Save Testing

## Description
Rapidly save and load 100 times. Verify no resource leaks, file handle leaks, or state corruption.

## Acceptance Criteria
- [ ] 100 save/load cycles without crash
- [ ] No file handle leaks (temp files cleaned up)
- [ ] Entity count stable after cycles
- [ ] Treasury value stable after cycles
