# UX-033: Improved Overlay Color Ramps

## Priority: T1 (Core Polish)
## Effort: Small (1-2 days)
## Source: camera_controls_ux.md -- Section 9: Enhanced Data Overlays

## Description
Current overlay colors may not use perceptually uniform color ramps. Implement scientifically-based color ramps (viridis, inferno, turbo) for continuous data, and categorical colors for discrete overlays.

## Acceptance Criteria
- [ ] Continuous overlays use perceptually uniform color ramp (e.g., viridis)
- [ ] Discrete overlays use distinguishable categorical colors
- [ ] Color ramps are colorblind-friendly
- [ ] Each overlay type has an appropriate ramp
