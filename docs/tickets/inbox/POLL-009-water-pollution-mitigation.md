# POLL-009: Water Pollution Mitigation Measures

## Priority: T3 (Differentiation)

## Description
Implement the 7 water pollution mitigation measures defined in the research doc: treatment plant upgrades, riparian buffers, constructed wetlands, stormwater detention, industrial pretreatment, sewer separation, and pervious pavement.

## Current State
- Water treatment plants exist (`WaterTreatmentPlant` service type) but only reduce pollution in a radius.
- No treatment level upgrades.
- No riparian buffer or wetland building types.
- No sewer separation concept.
- No pervious pavement road option.

## Definition of Done
- [ ] Treatment plant upgrade: +1 treatment level per upgrade action (costs $50K-200K).
- [ ] Riparian buffer zones: placeable along waterways, -40% NPS entering waterway.
- [ ] Constructed wetlands: 4x4 cell facility, -30% pollution passing through.
- [ ] Stormwater detention: delays and filters runoff per basin.
- [ ] Industrial pretreatment policy: -50% industrial discharge, policy + cost.
- [ ] Sewer separation: eliminates CSO events (very expensive).
- [ ] Pervious pavement: -60% road runoff pollution, 2x road cost.

## Test Plan
- [ ] Integration test: upgrading treatment level reduces downstream pollution.
- [ ] Integration test: riparian buffer reduces pollution at waterway.
- [ ] Integration test: pervious pavement roads produce less runoff pollution in rain.

## Pitfalls
- Many of these require new building types or road variants.
- Sewer separation requires the combined/separated sewer system (not yet implemented).
- Depends on NPS pollution system (POLL-007).

## Code References
- `crates/simulation/src/water_pollution.rs`
- `crates/simulation/src/services.rs`: `ServiceType`
- Research: `environment_climate.md` section 1.2.6
