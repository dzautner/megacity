# POLL-026: Temporal Amortization (Quadrant Rotation) for Pollution Updates

## Priority: T2 (Depth)

## Description
Spread air pollution dispersion updates across 4 ticks by rotating which quadrant of the grid is updated each tick. Each tick processes 1/4 of the grid (16,384 cells). Full grid updated every 4 ticks with each quadrant refreshed once. This quarters per-tick cost.

## Current State
- Full grid (65,536 cells) updated every pollution tick.
- No temporal amortization.

## Definition of Done
- [ ] Grid divided into 4 quadrants: (0,0)-(127,127), (128,0)-(255,127), (0,128)-(127,255), (128,128)-(255,255).
- [ ] `tick_counter % 4` determines which quadrant to process.
- [ ] Each quadrant gets full dispersion calculation on its turn.
- [ ] Decay applied to all cells every tick (O(n) multiply, cheap).
- [ ] Source contributions only applied to their quadrant's cells when it's that quadrant's turn.
- [ ] Visual result: smooth pollution map that refreshes fully every 4 ticks.

## Test Plan
- [ ] Unit test: quadrant 0 updated on tick 0, quadrant 1 on tick 1, etc.
- [ ] Performance test: per-tick cost is 1/4 of full update.
- [ ] Visual test: no noticeable quadrant boundaries or flickering.
- [ ] Integration test: total pollution values match non-amortized version within 5%.

## Pitfalls
- Sources near quadrant boundaries need to contribute to adjacent quadrants.
- Decay must still apply globally every tick to avoid visible quadrant aging differences.
- 4-tick latency for source changes is acceptable but must not cause visual artifacts.

## Code References
- `crates/simulation/src/pollution.rs`: `update_pollution`
- Research: `environment_climate.md` section 9.2.3
