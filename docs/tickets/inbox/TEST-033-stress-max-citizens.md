# TEST-033: Stress Test: Maximum Citizens (500K)

## Priority: T2 (Depth)
## Effort: Medium (2-3 days)
## Source: testing_strategy.md -- Section 6.1: Maximum Citizens

## Description
Spawn 500K citizens with roads and buildings. Run 100 ticks. Verify no crashes, panics, or OOM. Track tick time.

## Acceptance Criteria
- [ ] 500K citizens spawned with valid homes and jobs
- [ ] 100 ticks complete without crash
- [ ] No OOM (RSS stays under 4GB)
- [ ] Per-tick time measured and reported
- [ ] Tagged `#[ignore]` (slow test)
