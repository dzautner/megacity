# CIT-069: Urban Heat Island Effect

**Priority:** T3 (Differentiation)
**Complexity:** Low-Medium (1-2 person-weeks)
**Dependencies:** None
**Source:** master_architecture.md Section 1.11

## Description

Dense urban areas are warmer than surrounding areas due to concrete/asphalt heat absorption. Temperature modifier per cell: +1C per 20% impervious surface coverage within chunk. Trees and parks reduce heat island (-0.5C per 10% green coverage). Heat island effects: increased cooling energy demand in summer, health risk during heat waves (elderly vulnerable), reduced comfort. Green roofs policy reduces heat island by 30%.

## Definition of Done

- [ ] Per-cell temperature modifier from impervious surface
- [ ] Tree/park cooling effect
- [ ] Heat island temperature added to weather temperature
- [ ] Increased cooling demand in summer
- [ ] Health risk during heat waves (elderly mortality +5% at >40C)
- [ ] Green roof policy reduces heat island
- [ ] Heat island overlay visualization

## Test Plan

- Unit test: dense area is warmer than park area
- Unit test: trees reduce local temperature
- Unit test: heat wave + heat island = health risk
- Integration test: heat island visible in temperature overlay

## Pitfalls

- Heat island effect is subtle; make it visible through energy costs and occasional health events

## Relevant Code

- `crates/simulation/src/weather.rs` (Weather.temperature)
- `crates/simulation/src/trees.rs` (TreeGrid)
