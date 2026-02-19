# CSLN-001: Performance Budget and Frame Rate Target
**Priority:** T0
**Complexity:** M
**Dependencies:** none
**Source:** cities_skylines_analysis.md, section 16.7 (lesson 1)

## Description
Establish and enforce a performance budget. CS2's #1 failure was performance. Megacity must maintain 60fps at 100K citizens as a hard requirement. Every new system must be profiled and must not push frame time above budget.

- Target: 60fps (16.67ms frame budget) with 100K citizens
- Profile every new system before merge
- Per-system budgets: simulation < 8ms, rendering < 6ms, UI < 2ms
- Automated performance test: spawn 100K citizens, measure frame time
- CI gate: performance regression tests block merge
- Performance overlay (F3 key): show per-system timing in debug builds

## Definition of Done
- [ ] Performance target documented and agreed
- [ ] Per-system timing instrumented
- [ ] Automated performance test at 100K citizens
- [ ] Performance overlay in debug builds
- [ ] At least 3 performance regression tests in CI

## Test Plan
- Benchmark: 100K citizens at 60fps on reference hardware
- Regression: New system does not increase frame time by > 1ms

## Pitfalls
- Performance target may need adjustment based on hardware tier
- Profiling overhead itself should be negligible in release builds
- Some systems are OK being slow if they run on slow tick (not every frame)

## Relevant Code
- `crates/app/src/main.rs` -- Bevy configuration for frame rate
- All crates -- timing instrumentation
