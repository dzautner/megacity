# TRAF-003: Road Hierarchy Enforcement and Warnings
**Priority:** T2
**Complexity:** M
**Dependencies:** TRAF-002
**Source:** cities_skylines_analysis.md, section 6.3; master_architecture.md, section M3

## Description
Detect and warn when player violates road hierarchy principles. Local roads connecting directly to highways creates bottlenecks. Proper hierarchy: Local -> Collector -> Arterial -> Highway.

- Detect direct Local-to-Highway connections (bad)
- Detect missing collector roads between local and arterial
- Warn when residential street handles highway-level traffic
- Visual indicator: bottleneck warning icon at problem intersections
- Advisor suggestion: "This local road carries 5x its designed capacity. Consider upgrading to an Avenue."

## Definition of Done
- [ ] Road hierarchy violations detected
- [ ] Warning icons at problem intersections
- [ ] Advisor generates hierarchy suggestions
- [ ] Traffic overlay shows hierarchy mismatches

## Test Plan
- Integration: Connect local road directly to highway, verify warning appears
- Integration: Build proper hierarchy, verify no warnings

## Pitfalls
- Must not be too aggressive with warnings (player may have intentional design)
- Hierarchy is a guideline, not a hard rule
- Detection must consider road types of connecting segments at each intersection

## Relevant Code
- `crates/simulation/src/traffic.rs` -- hierarchy analysis
- `crates/simulation/src/advisors.rs` -- hierarchy advice
- `crates/rendering/src/status_icons.rs` -- bottleneck warning icons
