# TEST-025: Criterion Benchmark: Grid Operations

## Priority: T1 (Core)
## Effort: Small (0.5 day)
## Source: testing_strategy.md -- Section 5.1: criterion.rs Microbenchmarks

## Description
Benchmark grid operations: neighbors4 (center/corner), world_to_grid, grid_to_world. Baseline for regression detection.

## Acceptance Criteria
- [ ] Benchmark neighbors4 at center and corner
- [ ] Benchmark world_to_grid conversion
- [ ] Benchmark grid_to_world conversion
- [ ] All operations < 100ns
