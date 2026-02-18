# TEST-034: Save/Load Round-Trip Test

## Priority: T1 (Core)
## Effort: Medium (2-3 days)
## Source: testing_strategy.md -- Save Testing

## Description
Create a city with known state (roads, buildings, citizens, budget, services). Save. Load. Compare all critical state: grid cells, citizen count, treasury, building count, road network.

## Acceptance Criteria
- [ ] Create city with roads, zones, buildings, citizens, services
- [ ] Save to temp file
- [ ] Load from temp file
- [ ] Grid cells match (cell_type, zone, building_id, road_type)
- [ ] Citizen count matches
- [ ] Treasury matches
- [ ] Building count matches
- [ ] Road segment count matches
