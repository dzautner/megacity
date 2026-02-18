# DISASTER-022: Building Fire System Enhancement with Sprinklers and Materials

## Priority: T2 (Depth)

## Description
Enhance the building fire system with fire resistance materials, sprinkler systems, and construction type vulnerability. Currently `FireGrid` is a simple u8 with random ignition and service-based extinguishing. Add building material effects, sprinkler auto-suppression, and fire investigation causes.

## Current State
- `FireGrid` uses u8 per cell (0=no fire, >0 = intensity).
- Random ignition at 0.05% per building per update.
- Fire stations extinguish fires in service radius.
- No building material distinction.
- No sprinkler system.
- No fire cause variety.

## Definition of Done
- [ ] `FireResistance` component: material type (wood/concrete/steel), has_sprinklers, defensible_space.
- [ ] Ignition probability by material: wood=1.0x, mixed=0.5x, concrete=0.1x, steel=0.05x.
- [ ] Sprinkler system: auto-suppresses fire intensity by 80% within 1 tick, $5K/building.
- [ ] Fire causes: electrical (30%), cooking (25%), heating (20%), arson (10%), lightning (10%), industrial accident (5%).
- [ ] Fire spread between buildings: probability based on proximity, material, wind.
- [ ] Damage calculation: partial damage at low intensity, total loss at sustained high intensity.

## Test Plan
- [ ] Unit test: concrete building has 10% of wood building ignition probability.
- [ ] Unit test: sprinkler system reduces fire intensity by 80%.
- [ ] Unit test: fire spread probability decreases with building separation.
- [ ] Integration test: fire in wood building district spreads; concrete district does not.

## Pitfalls
- Must not break existing fire station extinguishing logic.
- Sprinkler auto-suppression should not make fire stations obsolete.
- Fire causes as a breakdown are informational; individual fires still use random ignition.

## Code References
- `crates/simulation/src/fire.rs`: `FireGrid`, fire systems
- Research: `environment_climate.md` section 5.3 (building materials)
