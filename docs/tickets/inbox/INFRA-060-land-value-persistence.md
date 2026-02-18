# INFRA-060: Land Value Persistence and Momentum
**Priority:** T1
**Complexity:** M (1-3 days)
**Dependencies:** none
**Source:** master_architecture.md, Section 6.2

## Description
Fix land value system to have persistence and gradual change instead of resetting to base 50 every cycle. Implement exponential moving average: `new_value = alpha * computed_value + (1 - alpha) * previous_value` with alpha ~0.1 (slow response). Add neighborhood spillover via diffusion (high-value cells raise neighbor values). Land value should have momentum: areas trending up continue trending up unless conditions change.

## Definition of Done
- [ ] Land value persists between update cycles (no reset to 50)
- [ ] Exponential smoothing with configurable alpha
- [ ] Neighborhood diffusion (per-cell value influenced by 8-neighbors)
- [ ] Land value changes are gradual, not instant
- [ ] Land value serialized in save files
- [ ] Tests pass

## Test Plan
- Unit: Placing a service building gradually increases nearby land value over multiple cycles
- Unit: Removing service gradually decreases land value (not instantly)
- Integration: Downtown area maintains high land value even during temporary disruptions

## Pitfalls
- Alpha too low = land value never changes; too high = no smoothing
- Diffusion can cause runaway values if not capped
- Need to handle initialization (what is previous_value on first tick?)

## Relevant Code
- `crates/simulation/src/land_value.rs` -- land value computation (currently resets to 50)
- `crates/save/src/serialization.rs` -- save land value grid
