# SVC-017: Water Treatment and Quality

**Priority:** T2 (Depth)
**Complexity:** Medium (1-2 person-weeks)
**Dependencies:** None
**Source:** master_architecture.md Section 1.12, historical_demographics_services.md

## Description

Water treatment plant processes sewage and contaminated water. Treatment quality depends on plant capacity and funding. Untreated sewage pollutes water sources. Treatment levels: primary (removes solids, 60% pollutant removal), secondary (biological, 85%), tertiary (advanced, 95%). WaterTreatmentPlant and WellPump already in ServiceType. Insufficient treatment causes water pollution -> health issues -> cholera risk.

## Definition of Done

- [ ] Treatment level tracking (primary/secondary/tertiary)
- [ ] Treatment capacity vs demand
- [ ] Untreated overflow when demand > capacity
- [ ] Treated water reduces water pollution in area
- [ ] Treatment level affects removal percentage (60/85/95%)
- [ ] WellPump provides clean water in low-pollution areas
- [ ] Water quality metric per area

## Test Plan

- Unit test: tertiary treatment removes 95% pollution
- Unit test: overflow at capacity causes water pollution spike
- Unit test: well pump in clean area provides water

## Pitfalls

- Water treatment interacts with water pollution grid; ensure consistency

## Relevant Code

- `crates/simulation/src/services.rs` (WaterTreatmentPlant, WellPump)
- `crates/simulation/src/water_pollution.rs`
- `crates/simulation/src/groundwater.rs`
