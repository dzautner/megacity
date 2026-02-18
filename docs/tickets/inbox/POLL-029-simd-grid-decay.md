# POLL-029: SIMD-Optimized Grid Decay Operations

## Priority: T3 (Differentiation)

## Description
Optimize grid-wide decay/multiply operations using SIMD (AVX2 or NEON). Decay operations (`grid[i] *= factor`) touch all 65,536 cells and are perfect candidates for SIMD processing (8 f32 values at once). Expected speedup: 4-8x for decay operations.

## Current State
- Grid decay uses scalar multiplication in a for loop.
- No SIMD optimization.
- Grid operations are already fast but will scale with more grids.

## Definition of Done
- [ ] `decay_grid(data: &mut [f32], factor: f32)` using `chunks_exact_mut(8)` for autovectorization.
- [ ] Clamp-and-zero: values below `MIN_THRESHOLD` snapped to 0.0 to avoid denormalized floats.
- [ ] Applied to: air pollution, water pollution, noise, soil contamination, UHI grids.
- [ ] Benchmark: measure speedup vs scalar version.
- [ ] Fallback: scalar path for non-SIMD targets (WASM, etc.).

## Test Plan
- [ ] Unit test: SIMD decay produces identical results to scalar.
- [ ] Benchmark: SIMD path is 3x+ faster than scalar.
- [ ] Integration test: all grid values decay correctly over time.

## Pitfalls
- Rust autovectorization may already optimize `chunks_exact_mut`; explicit SIMD may not help.
- Denormalized floats near zero cause performance collapse; must snap to zero.
- Cross-platform: different SIMD support on x86 (AVX2) vs ARM (NEON).

## Code References
- `crates/simulation/src/pollution.rs`: grid decay
- Research: `environment_climate.md` section 9.2.2
