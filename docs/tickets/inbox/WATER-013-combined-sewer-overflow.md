# WATER-013: Combined Sewer Overflow (CSO) Events

## Priority: T3 (Differentiation)

## Description
Implement combined sewer overflow events where heavy rain causes mixed stormwater + sewage to overflow into water bodies. Combined sewers handle both sewage and stormwater; during heavy rain they exceed capacity and discharge untreated sewage.

## Current State
- No sewer type distinction (combined vs separated).
- No CSO events.
- No rain-triggered sewage overflow.

## Definition of Done
- [ ] Sewer type policy: Combined (default, cheap) vs Separated (expensive, no CSO).
- [ ] CSO trigger: rainfall intensity > combined sewer capacity.
- [ ] CSO discharge: untreated sewage enters nearest water body.
- [ ] Water pollution spike during CSO events.
- [ ] CSO frequency tracking for environmental compliance.
- [ ] Sewer separation upgrade: very expensive ($500K/cell), eliminates CSO permanently.
- [ ] Green infrastructure (rain gardens, permeable pavement) reduces CSO frequency.

## Test Plan
- [ ] Unit test: heavy rain in combined sewer area triggers CSO.
- [ ] Unit test: separated sewer area has no CSO.
- [ ] Integration test: CSO causes water pollution spike.
- [ ] Integration test: green infrastructure reduces CSO frequency.

## Pitfalls
- Sewer infrastructure upgrade is extremely expensive; needs to feel justified.
- CSO is a real-world problem but may be too technical for casual players.
- Depends on WATER-007 (wastewater collection) and WATER-004 (stormwater).

## Code References
- `crates/simulation/src/water_pollution.rs`: pollution source
- Research: `environment_climate.md` section 2.5
