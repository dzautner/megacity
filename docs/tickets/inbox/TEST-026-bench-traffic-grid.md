# TEST-026: Criterion Benchmark: Traffic Grid Operations

## Priority: T1 (Core)
## Effort: Small (0.5 day)
## Source: testing_strategy.md -- Section 5.1: criterion.rs Microbenchmarks

## Description
Benchmark traffic grid: full_clear 256x256, congestion_lookup, path_cost_with_road. Baseline for regression detection.

## Acceptance Criteria
- [ ] Benchmark full grid clear
- [ ] Benchmark congestion level lookup
- [ ] Benchmark path cost calculation with road type
- [ ] Budget: traffic grid update < 2ms for 100K citizens
