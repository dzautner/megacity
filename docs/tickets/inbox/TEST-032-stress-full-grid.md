# TEST-032: Stress Test: Full Grid Saturation

## Priority: T2 (Depth)
## Effort: Small (1 day)
## Source: testing_strategy.md -- Section 6.1: Maximum Load Scenarios

## Description
Fill entire 256x256 grid: every other row = roads, all other cells = ResidentialHigh with power/water. Build CSR graph. Verify pathfinding still works across full grid.

## Acceptance Criteria
- [ ] Full grid filled with alternating roads and zoned cells
- [ ] CSR graph builds successfully
- [ ] node_count > 0 and edge_count > 0
- [ ] Cross-map pathfinding returns valid path
- [ ] No panics or OOM
