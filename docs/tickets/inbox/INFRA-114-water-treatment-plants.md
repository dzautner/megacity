# INFRA-114: Water Treatment Plant Tiers and Quality
**Priority:** T2
**Complexity:** M (1-3 days)
**Dependencies:** INFRA-019
**Source:** infrastructure_engineering.md, Section 3

## Description
Implement multi-stage water treatment plants as upgradeable facilities. Stages: Screening/Coagulation (basic), Sedimentation+Filtration (standard), Disinfection+pH (full treatment). Each stage improves water quality. Skipping stages means lower water quality -> health events. Source water quality depends on source type (groundwater > river > lake). Treatment capacity limits (gallons per day). Drought management with staged restrictions when reservoir < 75/50/30%.

## Definition of Done
- [ ] Water treatment plant building with upgradeable stages
- [ ] Water quality metric affected by treatment level
- [ ] Source type affects required treatment
- [ ] Treatment capacity limits
- [ ] Drought management stages
- [ ] Water quality overlay
- [ ] Tests pass

## Test Plan
- Unit: Full treatment produces quality 100%; partial treatment produces lower quality
- Unit: Drought stage 2 triggers at 50% reservoir level
- Integration: Low water quality causes citizen health problems

## Pitfalls
- Water quality system needs interaction with health system
- Reservoir level tracking needs weather/seasonal integration
- Treatment plant capacity must match city water demand

## Relevant Code
- `crates/simulation/src/utilities.rs` -- water system
- `crates/simulation/src/health.rs` -- water quality health effects
