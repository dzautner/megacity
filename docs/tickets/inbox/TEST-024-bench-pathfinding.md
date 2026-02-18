# TEST-024: Criterion Benchmark: Pathfinding

## Priority: T1 (Core)
## Effort: Small (1-2 days)
## Source: testing_strategy.md -- Section 5.1: criterion.rs Microbenchmarks

## Description
Add criterion benchmarks for CSR A* pathfinding: short (10 cells), medium (50), long (200), cross-map (248). Baseline for regression detection.

## Acceptance Criteria
- [ ] `criterion` crate added as dev dependency
- [ ] Benchmarks: short_10, medium_50, long_200, cross_map
- [ ] Grid road network (every 8 cells) as test fixture
- [ ] Results reported with statistical confidence
- [ ] Budget: single A* < 1ms
