# TEST-005: Unit Tests for Traffic Congestion Calculation

## Priority: T1 (Core)
## Effort: Small (0.5 day)
## Source: testing_strategy.md -- Section 2: Unit Testing

## Description
Test congestion level calculation, BPR travel time function, and traffic grid operations. Verify congestion in [0.0, 1.0].

## Acceptance Criteria
- [ ] Test congestion_level with various density values
- [ ] Test zero density = zero congestion
- [ ] Test max density = congestion 1.0
- [ ] Test BPR function: t = t0 * (1 + alpha * (v/c)^beta)
- [ ] Verify congestion always in [0.0, 1.0]
