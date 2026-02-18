# WATER-003: Water Treatment Plant Level System

## Priority: T2 (Depth)

## Description
Implement tiered water treatment with upgradeable treatment levels. Currently `WaterTreatmentPlant` just reduces pollution in a radius. The research doc specifies 5 treatment levels (None, Primary, Secondary, Tertiary, Advanced) with distinct effectiveness percentages, costs, and output quality.

## Current State
- `ServiceType::WaterTreatmentPlant` exists and reduces groundwater pollution in radius 12.
- No treatment level concept.
- No treatment cost scaling.
- No effluent quality tracking.

## Definition of Done
- [ ] `TreatmentLevel` enum: None(0%), Primary(60%), Secondary(85%), Tertiary(95%), Advanced(99%).
- [ ] Treatment plants have upgradeable level (start at Primary, upgrade costs $50K-200K per level).
- [ ] Effluent quality depends on treatment level and input water quality.
- [ ] Treatment cost per million gallons scales with level: Primary=$1K, Secondary=$2K, Tertiary=$5K, Advanced=$10K.
- [ ] Treatment plant capacity (MGD) limits how much water can be processed.
- [ ] Drinking water quality affects citizen health (untreated water = disease risk).

## Test Plan
- [ ] Unit test: treatment effectiveness matches specified percentages.
- [ ] Unit test: upgrade cost calculation is correct per level.
- [ ] Integration test: upgrading treatment improves downstream water quality.
- [ ] Integration test: treatment plant at capacity rejects overflow.

## Pitfalls
- Building upgrade UI does not exist; needs a building-specific action panel.
- Treatment level must be serialized for save/load.
- Treatment capacity must match water demand or excess goes untreated.

## Code References
- `crates/simulation/src/services.rs`: `ServiceType::WaterTreatmentPlant`
- `crates/simulation/src/groundwater.rs`: purification in `update_groundwater`
- Research: `environment_climate.md` section 2.2
