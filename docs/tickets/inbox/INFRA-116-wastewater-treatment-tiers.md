# INFRA-116: Wastewater Treatment Plant Tiers
**Priority:** T2
**Complexity:** M (1-3 days)
**Dependencies:** INFRA-020
**Source:** infrastructure_engineering.md, Section 4

## Description
Implement wastewater treatment plants with upgradeable stages: Preliminary (screens, grit removal), Primary (sedimentation), Secondary (biological treatment, 85% BOD removal), Tertiary (nutrient removal, disinfection, enables water reuse). Primary-only treatment pollutes receiving waters. Biogas from anaerobic digestion offsets plant energy costs. Treatment capacity limits. Combined sewer overflow events during heavy rain.

## Definition of Done
- [ ] Wastewater treatment plant with 4 upgradeable stages
- [ ] Each stage improves effluent quality
- [ ] Insufficient treatment causes water pollution
- [ ] Biogas energy recovery from secondary+ treatment
- [ ] CSO events during heavy rain (combined sewer systems)
- [ ] Treatment capacity limits
- [ ] Tests pass

## Test Plan
- Unit: Primary-only treatment produces high BOD effluent -> water pollution
- Unit: Biogas from secondary treatment offsets 20% of plant energy cost
- Unit: CSO event triggers when rain + sewage exceeds capacity

## Pitfalls
- CSO events need combined vs separate sewer distinction
- Biogas revenue is small but creates interesting optimization
- Effluent must discharge to water body (river/ocean)

## Relevant Code
- `crates/simulation/src/utilities.rs` -- sewer/treatment system
- `crates/simulation/src/water_pollution.rs` -- effluent quality
- `crates/simulation/src/pollution.rs` -- environmental impact
