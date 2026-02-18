# POLL-014: Soil Remediation Building and Phytoremediation

## Priority: T3 (Differentiation)

## Description
Implement soil remediation options: excavation (expensive, fast), bioremediation (moderate, slow), phytoremediation (cheap, very slow), and containment (prevents spread only). These are the only ways to clean up soil contamination given its near-zero natural decay.

## Current State
- No remediation system exists.
- No brownfield redevelopment concept.
- No remediation building type.

## Definition of Done
- [ ] Remediation methods: Excavation (-10/tick, $500/cell), Bioremediation (-3/tick, $150/cell), Phytoremediation (-0.5/tick, $30/cell), Containment (stops spread, $80/cell).
- [ ] Remediation building type placeable on contaminated cells.
- [ ] Brownfield indicator: cells with contamination > 20 shown on overlay.
- [ ] Post-remediation: cell becomes buildable again when contamination < 10.
- [ ] Health effects: citizens on contaminated soil (>30) suffer health penalty.
- [ ] Land value: contaminated soil reduces land value by up to -60%.

## Test Plan
- [ ] Unit test: excavation removes contamination at 10 units per tick.
- [ ] Unit test: phytoremediation is slower but cheaper.
- [ ] Integration test: contaminated cell becomes buildable after remediation completes.
- [ ] Integration test: citizens health improves after remediation.

## Pitfalls
- Depends on POLL-013 (soil contamination grid).
- Remediation timescale must be meaningful but not frustrating (game-years, not real-time years).
- Player may not realize they need to remediate before building.

## Code References
- Research: `environment_climate.md` sections 1.4.3, 1.4.4
- `crates/simulation/src/services.rs`: new service type needed
