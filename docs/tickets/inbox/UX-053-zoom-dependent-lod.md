# UX-053: Seamless Zoom-Dependent LOD Transitions

## Priority: T2 (Depth)
## Effort: Medium (3-5 days)
## Source: camera_controls_ux.md -- Section 8: Zoom LOD Tiers

## Description
Smooth transitions between LOD tiers as camera zooms. Currently uses Full/Simplified/Abstract. Add cross-fade or morphing between tiers to avoid pop-in.

## Acceptance Criteria
- [ ] LOD transitions smooth (no visible pop-in)
- [ ] Cross-fade alpha blending during transition
- [ ] Hysteresis: different zoom thresholds for switching up vs down
- [ ] Performance: no double-rendering during transition
