# DISASTER-003: Earthquake Secondary Effects (Fire, Liquefaction, Aftershocks)

## Priority: T3 (Differentiation)

## Description
Implement secondary earthquake effects: post-earthquake fires from gas line breaks, soil liquefaction in soft/fill areas, landslides on steep terrain, and aftershock sequences following the main event (Omori's law).

## Current State
- No post-earthquake fire ignition.
- No liquefaction system.
- No aftershock sequences.
- No terrain slope effects.

## Definition of Done
- [ ] Post-earthquake fires: MMI VII=5% chance per cell, MMI IX=20% chance. Sets `OnFire` component on buildings.
- [ ] Broken water mains impede firefighting (fire coverage reduced during earthquake).
- [ ] Liquefaction: soft soil cells at MMI>7 have 30% chance of building collapse regardless of construction type.
- [ ] Landslides: steep cells at MMI>6 roll for landslide, burying downhill cells.
- [ ] Aftershocks: magnitude = main_magnitude - 1.2, frequency decreases exponentially over 3-7 game-days.
- [ ] Infrastructure disruption: roads 10% closure, water mains 20% break, power lines 15% failure at MMI>VII.

## Test Plan
- [ ] Unit test: fire ignition probability at MMI IX = 20%.
- [ ] Unit test: aftershock magnitude is approximately main - 1.2.
- [ ] Integration test: M7 earthquake triggers fires and liquefaction in susceptible areas.
- [ ] Integration test: aftershocks cause additional damage over following days.

## Pitfalls
- Depends on DISASTER-001 (MMI system) and DISASTER-002 (construction types).
- Aftershock sequences need a scheduling mechanism (queue of future events).
- Liquefaction requires soil type data (not currently per-cell).

## Code References
- `crates/simulation/src/disasters.rs`: earthquake processing
- `crates/simulation/src/fire.rs`: `OnFire` component, fire ignition
- Research: `environment_climate.md` section 5.1.4
