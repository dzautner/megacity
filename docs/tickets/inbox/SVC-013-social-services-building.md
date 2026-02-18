# SVC-013: Social Services Building Types

**Priority:** T2 (Depth)
**Complexity:** Low (0.5 person-weeks)
**Dependencies:** None
**Source:** historical_demographics_services.md Section 5.4

## Description

Add social services building types: WelfareOffice (already in ServiceType), CommunityCenter, SubstanceAbuseTreatmentCenter, SeniorCenter, YouthCenter. Each provides coverage radius for specific demographic groups. WelfareOffice: welfare eligibility processing. CommunityCenter: social need satisfaction, community events. SubstanceAbuse: reduces addiction-related crime and health issues. SeniorCenter: elderly happiness bonus. YouthCenter: reduces juvenile crime.

## Definition of Done

- [ ] CommunityCenter service type added
- [ ] SubstanceAbuseTreatmentCenter service type
- [ ] SeniorCenter service type
- [ ] YouthCenter service type
- [ ] Each provides coverage with demographic-specific effects
- [ ] CommunityCenter: +5 happiness, social need boost
- [ ] YouthCenter: -15% juvenile crime in radius
- [ ] SeniorCenter: +10 happiness for retired citizens in radius

## Test Plan

- Unit test: community center improves social satisfaction
- Unit test: youth center reduces local crime
- Integration test: senior center improves elderly happiness

## Pitfalls

- Many service types already exist; avoid feature bloat with marginal buildings

## Relevant Code

- `crates/simulation/src/services.rs` (ServiceType enum)
