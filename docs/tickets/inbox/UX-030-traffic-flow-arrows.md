# UX-030: Traffic Flow Arrows Overlay

## Priority: T2 (Depth)
## Effort: Medium (2-3 days)
## Source: camera_controls_ux.md -- Section 9.4: Animated Directional Overlays

## Description
When traffic overlay is active, spawn moving arrow glyphs on road cells. Arrow direction follows traffic flow, opacity proportional to volume, color from green (free) to red (congested).

## Acceptance Criteria
- [ ] Animated arrows on road cells during traffic overlay
- [ ] Arrow direction matches traffic flow direction
- [ ] Arrow opacity proportional to traffic volume
- [ ] Arrow color: green (free flow) to red (congested)
- [ ] Performance: arrows LOD'd at far zoom
