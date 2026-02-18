# SAVE-029: Serialize District Data and DistrictMap

## Priority: T1 (Medium-Term)
## Effort: Small (1 day)
## Source: save_system_architecture.md -- Current Serialization Audit

## Description
Verify Districts, DistrictMap, and per-district policies/names persist across save/load.

## Acceptance Criteria
- [ ] District boundaries roundtrip correctly
- [ ] District names persist
- [ ] Per-district settings (if any) persist
- [ ] DistrictMap grid data matches pre-save
