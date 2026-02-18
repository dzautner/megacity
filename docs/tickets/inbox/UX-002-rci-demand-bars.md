# UX-002: RCI Demand Bars in Toolbar

## Priority: T1 (Core Polish)
## Effort: Small (1 day)
## Source: camera_controls_ux.md -- Section 13.1: HUD

## Description
Display Residential/Commercial/Industrial demand as colored bars in the top toolbar. The `ZoneDemand` resource exists but is not displayed (parameter has `_demand` underscore prefix in toolbar.rs).

## Acceptance Criteria
- [ ] Three vertical bars: R (green), C (blue), I (yellow)
- [ ] Positive demand shown as bar growing up
- [ ] Negative demand (surplus) shown as bar growing down or red
- [ ] Tooltip on hover shows exact demand values
- [ ] Prominent position near population count
