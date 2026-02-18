# GOV-009: District-Level Policy Settings

**Priority:** T2 (Depth)
**Complexity:** Medium (2-3 person-weeks)
**Dependencies:** None
**Source:** master_architecture.md Section 1.15, Section 3 T2

## Description

Allow per-district policy overrides. Each district can set: local tax rate modifier (+/- 20%), service budget allocation, specific policy toggles (parking limits, noise ordinance, historic preservation). District-level policies allow differentiated governance across city areas. Downtown can have higher taxes + better services. Suburbs can have lower taxes + fewer regulations.

## Definition of Done

- [ ] Per-district policy overrides stored in Districts resource
- [ ] Tax rate modifier per district (+/- 20%)
- [ ] Service budget weight per district
- [ ] District-specific policy toggles (3-5 policies)
- [ ] District policies affect local simulation (crime, happiness, land value)
- [ ] UI for editing district policies
- [ ] District comparison view showing policy differences

## Test Plan

- Unit test: district tax modifier applied correctly
- Unit test: district policy overrides city-level default
- Integration test: two adjacent districts with different policies show different outcomes

## Pitfalls

- Per-district policies multiply system complexity; keep list of per-district options small
- District boundary changes should preserve policy settings

## Relevant Code

- `crates/simulation/src/districts.rs` (Districts, DistrictMap)
- `crates/simulation/src/policies.rs` (Policies)
