# CIT-077: City Specialization System Enhancement

**Priority:** T2 (Depth)
**Complexity:** Low-Medium (1 person-week)
**Dependencies:** None
**Source:** master_architecture.md Section 1.8

## Description

City specialization provides economic bonuses when focusing on specific sectors. Specializations: Industrial (manufacturing efficiency), Tourism (attraction bonus), Education (research), Tech (office productivity), Trade (import/export discounts), Resource (extraction efficiency). Specialization unlocked when sector exceeds 25% of economy. Multiple specializations possible at reduced bonus. Specialization provides unique buildings and policies.

## Definition of Done

- [ ] Specialization detection from economic mix
- [ ] Bonus application per specialization type
- [ ] Multiple specializations at reduced bonus (each)
- [ ] Specialization-specific buildings unlocked
- [ ] Specialization policies unlocked
- [ ] Specialization panel in economy UI
- [ ] Specialization affects immigration (attracts matching workers)

## Test Plan

- Unit test: 30% tourism economy unlocks tourism specialization
- Unit test: specialization bonus applies correctly
- Unit test: multiple specializations reduce each bonus

## Pitfalls

- Already partially implemented in specialization.rs; extend

## Relevant Code

- `crates/simulation/src/specialization.rs` (CitySpecializations, SpecializationBonuses)
