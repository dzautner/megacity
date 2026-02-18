# INFRA-117: Green Infrastructure for Stormwater
**Priority:** T3
**Complexity:** M (1-3 days)
**Dependencies:** INFRA-080
**Source:** infrastructure_engineering.md, Section 5

## Description
Implement green infrastructure types for stormwater management: rain gardens/bioretention (40-90% runoff reduction), bioswales, permeable pavement (50-90% reduction), green roofs (25-80% reduction), detention basins, constructed wetlands. Each is a placeable element reducing flood risk in its zone. Green infrastructure handles frequent small storms well but not 100-year events. Hybrid green+gray strategies needed.

## Definition of Done
- [ ] At least 4 green infrastructure building types
- [ ] Per-type runoff reduction percentage
- [ ] Placement tool and cost per type
- [ ] Stormwater risk reduction in affected area
- [ ] Environmental/aesthetic benefits (land value bonus)
- [ ] Tests pass

## Test Plan
- Unit: Rain garden in zone reduces runoff by 40-90%
- Unit: Green infrastructure does not prevent 100-year flood
- Integration: Green infrastructure visually appears in city and reduces flood events

## Pitfalls
- Multiple green infrastructure in same area: diminishing returns, not additive
- Green infrastructure has maintenance costs
- Permeable pavement replaces standard road surface -- special handling

## Relevant Code
- `crates/simulation/src/buildings.rs` -- infrastructure building types
- `crates/simulation/src/groundwater.rs` -- infiltration
