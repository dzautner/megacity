# TEST-027: Criterion Benchmark: Full Simulation Tick at Scale

## Priority: T1 (Core)
## Effort: Medium (2-3 days)
## Source: testing_strategy.md -- Section 5.2: Macro Benchmarks

## Description
Benchmark full simulation tick at 1K, 10K, 50K, 100K citizens. Budget: < 16ms at 100K citizens.

## Acceptance Criteria
- [ ] `create_benchmark_app()` helper spawns N citizens with road/building infrastructure
- [ ] Benchmark at 1K, 10K, 50K, 100K citizen counts
- [ ] 30-second measurement time, 10 samples
- [ ] Budget: full tick < 16ms at 100K citizens
