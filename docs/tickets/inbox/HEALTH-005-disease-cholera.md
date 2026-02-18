# HEALTH-005: Disease -- Cholera (Waterborne)

**Priority:** T3 (Differentiation)
**Complexity:** Low (included in HEALTH-002)
**Dependencies:** HEALTH-002 (SIR model)
**Source:** social_agent_simulation.md Section 9.1

## Description

Cholera: transmission via contaminated water. Triggered when WaterPollutionGrid exceeds threshold near water treatment intake. Transmission_rate 0.2 (only in contaminated zones), recovery_rate 0.15, mortality_rate 0.01 (1% without treatment, 0.1% with hospital). Incubation 1-5 days. Prevents: clean water infrastructure, water treatment plants. Cholera is historically significant (John Snow's discovery of waterborne disease).

## Definition of Done

- [ ] Cholera disease variant
- [ ] Triggered by water pollution exceeding threshold
- [ ] Spatial spread limited to water-pollution-affected zones
- [ ] Treatment reduces mortality from 1% to 0.1%
- [ ] Water treatment plant prevents outbreak
- [ ] Event notification: "Cholera outbreak detected"
- [ ] Boil water advisory effect (reduces spread)

## Test Plan

- Unit test: cholera only spreads in polluted water zones
- Unit test: treatment reduces mortality 10x
- Unit test: water treatment prevents outbreak
- Integration test: sewage near water intake triggers cholera

## Pitfalls

- Cholera requires water pollution system to be meaningful
- Outbreak severity must be dramatic enough to motivate water infrastructure investment

## Relevant Code

- HEALTH-002 infrastructure
- `crates/simulation/src/water_pollution.rs` (WaterPollutionGrid)
