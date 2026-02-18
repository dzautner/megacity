# UX-010: Selection Visual Feedback (Outline Glow + Pulsing)

## Priority: T1 (Core Polish)
## Effort: Small (1-2 days)
## Source: camera_controls_ux.md -- Section 10.2: Selection Visual Feedback

## Description
Selected entities need clear visual feedback. Render a bright outline (5% scaled-up mesh in highlight color) with pulsing alpha (0.3-0.6 at 2Hz). Show connected entity highlights (residents, workplace, coverage radius).

## Acceptance Criteria
- [ ] Selection highlight: 5% scaled-up mesh in highlight color
- [ ] Pulsing animation (alpha 0.3-0.6 at 2Hz)
- [ ] Connected highlights: residents, workplace, coverage radius
- [ ] Service buildings show coverage circle when selected
