# CIT-025: Tick Staggering Framework

**Priority:** T1 (Core -- performance)
**Complexity:** Low-Medium (1 person-week)
**Dependencies:** None
**Source:** social_agent_simulation.md Section 15.3

## Description

Formalize citizen processing distribution across tick slots. Instead of processing all citizens every N ticks (spike frame), process 1/N of citizens every tick (smooth frame). Utility function `citizen_tick_slot(entity, num_slots, tick)` distributes citizens by entity index modulo. Apply to: happiness (10 slots), health (10 slots), needs (5 slots), crime (5 slots), education (50 slots), lifecycle (365 slots).

## Definition of Done

- [ ] `citizen_tick_slot(entity, num_slots, tick) -> bool` utility function
- [ ] `update_happiness` uses staggered slots (10)
- [ ] `update_needs` uses staggered slots (5)
- [ ] `update_health` uses staggered slots (10)
- [ ] Documentation of system frequencies in code comments
- [ ] No observable gameplay difference vs non-staggered
- [ ] Frame time variance reduced by >50%

## Test Plan

- Unit test: citizen_tick_slot distributes evenly across slots
- Performance test: frame time variance measured before/after
- Integration test: happiness values identical (within float error) vs non-staggered

## Pitfalls

- Entity indices may cluster if spawned in bursts; use hash instead of modulo
- Staggering changes observation frequency; systems that expect every-tick updates break

## Relevant Code

- `crates/simulation/src/happiness.rs` (update_happiness)
- `crates/simulation/src/life_simulation.rs` (update_needs, update_health)
- `crates/simulation/src/lib.rs` (TickCounter, SlowTickTimer)
