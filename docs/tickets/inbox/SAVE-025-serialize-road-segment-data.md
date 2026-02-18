# SAVE-025: Serialize Full Road Segment Data

## Priority: T1 (Short-Term Fix)
## Effort: Small (1 day)
## Source: save_system_architecture.md -- Current Serialization Audit

## Description
Verify all RoadSegmentStore data roundtrips correctly including Bezier control points, segment IDs, intersection data, and one-way flags. Currently segments are serialized but intersection topology may be lost.

## Acceptance Criteria
- [ ] All Bezier control points (p0, p1, p2, p3) serialized
- [ ] Segment IDs preserved or deterministically regenerated
- [ ] Intersection references valid after load
- [ ] One-way flags serialized
- [ ] Road render matches pre-save state
