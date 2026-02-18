# SAVE-030: Save/Load Performance Benchmark at Scale

## Priority: T1 (Medium-Term)
## Effort: Small (1-2 days)
## Source: save_system_architecture.md -- Performance Testing

## Description
Create performance benchmarks for save/load at 10K, 50K, 100K, and 500K citizens. Target: snapshot <16ms, encode <500ms, save to disk <1s. Load <3s.

## Acceptance Criteria
- [ ] Benchmark script creates cities at 10K/50K/100K/500K population
- [ ] Save and load times measured and reported
- [ ] Performance budget: snapshot <16ms, full save <1s, full load <3s
- [ ] Results tracked in CI for regression detection
