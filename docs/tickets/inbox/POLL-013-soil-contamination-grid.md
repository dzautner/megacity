# POLL-013: Soil Contamination Grid and Persistence Model

## Priority: T2 (Depth)

## Description
Implement a soil contamination grid with near-permanent persistence. Unlike air/water pollution which decays rapidly, soil contamination persists for decades. The research doc specifies a `SoilContaminationGrid` with DECAY_RATE=0.9999 (near-permanent), lateral spread when concentration > 50, and remediation as the only realistic cleanup mechanism.

## Current State
- No soil contamination system exists.
- Ground pollution is only modeled through land value penalties.
- No historical contamination from demolished industrial buildings.
- No remediation options.

## Definition of Done
- [ ] `SoilContaminationGrid` (f32 per cell, 0-500 range) resource added.
- [ ] Industrial sources: 3.0 base rate per building level.
- [ ] Landfill sources: lined=1.0, unlined=5.0.
- [ ] Gas stations: 2.0 (fuel leaks).
- [ ] Natural decay: `SOIL_NATURAL_DECAY = 0.9999` per update (virtually permanent).
- [ ] Lateral spread: cells > 50 spread 0.01 to neighbors.
- [ ] Industrial cleanup: demolished industrial site retains contamination.
- [ ] Update frequency: every 30 ticks.
- [ ] Integration with groundwater: soil contamination seeps into groundwater quality.

## Test Plan
- [ ] Unit test: soil contamination decays less than 0.1% per update cycle.
- [ ] Unit test: lateral spread only occurs above threshold of 50.
- [ ] Integration test: demolished factory leaves persistent soil contamination.
- [ ] Integration test: contaminated soil degrades nearby groundwater.

## Pitfalls
- Near-zero decay means contamination is effectively permanent without remediation.
- Players may not understand why land remains unusable after demolishing a factory.
- Need clear UI indicator (brownfield overlay) to communicate contaminated areas.

## Code References
- Research: `environment_climate.md` sections 1.4.1-1.4.4
- `crates/simulation/src/pollution.rs`: existing air pollution grid (pattern to follow)
- `crates/simulation/src/groundwater.rs`: integration target
