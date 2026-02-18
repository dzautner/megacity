# SAVE-026: Post-Load Derived State Rebuild System

## Priority: T1 (Short-Term Fix)
## Effort: Medium (2-3 days)
## Source: save_system_architecture.md -- Future Architecture Recommendations

## Description
After loading, rebuild all derived state: CSR road graph from RoadNetwork, spatial indices from DestinationCache, service coverage grids from service buildings, traffic grid from citizen positions. This ensures consistency and allows removing derived state from save files.

## Acceptance Criteria
- [ ] CSR graph rebuilt from RoadNetwork/RoadSegmentStore on load
- [ ] SpatialIndex rebuilt from citizen/building positions
- [ ] ServiceCoverageGrid recalculated from service buildings
- [ ] TrafficGrid recalculated (or zeroed out)
- [ ] Explicit `post_load_rebuild()` system runs after `handle_load`
