# UX-020: Road Freehand Drawing Mode

## Priority: T3 (Differentiation)
## Effort: Large (1-2 weeks)
## Source: camera_controls_ux.md -- Section 12.2: Road Drawing Modes

## Description
Hold mouse and draw freehand path. System fits Bezier curves to drawn path using curve fitting algorithm. Produces organic, realistic road layouts.

## Acceptance Criteria
- [ ] Hold-and-draw gesture captures mouse path
- [ ] Path sampled at regular intervals
- [ ] Bezier curve fitting applied to sample points
- [ ] Resulting road segments placed along fitted curves
- [ ] Error tolerance configurable
