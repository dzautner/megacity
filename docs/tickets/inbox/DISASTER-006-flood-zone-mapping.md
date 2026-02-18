# DISASTER-006: Pre-Computed Flood Zone Mapping

## Priority: T2 (Depth)

## Description
Pre-compute flood risk zones (100-year and 500-year flood plains) based on terrain and river locations. Show these zones as an overlay to help players plan. Policies can prohibit building in high-risk zones or require flood insurance.

## Current State
- No flood zone mapping.
- No building restriction based on flood risk.
- No flood insurance concept.

## Definition of Done
- [ ] `FloodZoneGrid` resource with HIGH_RISK (100-year), MODERATE (500-year), LOW_RISK tiers.
- [ ] Computed at map generation by simulating 100-year rainfall (6 inches in 24 hours).
- [ ] Recomputed when terrain changes (levee construction, dam placement).
- [ ] Overlay showing flood zones in rendering.
- [ ] Policy: "Floodplain Regulation" -- prohibit building in HIGH_RISK zones.
- [ ] Policy: "Flood Insurance Mandate" -- required for MODERATE zones, costs 2% property value/year.
- [ ] Development with elevation requirements: allow building if foundation raised.

## Test Plan
- [ ] Unit test: low-elevation cells near rivers are HIGH_RISK.
- [ ] Unit test: high-elevation cells are LOW_RISK.
- [ ] Integration test: floodplain regulation prevents building placement in high-risk zones.
- [ ] Integration test: flood insurance policy costs appear in budget.

## Pitfalls
- 100-year simulation at map load is computationally expensive; can be simplified.
- Flood zones change with levee construction, requiring recomputation.
- Insurance cost tracking needs integration with budget/economy system.

## Code References
- `crates/simulation/src/grid.rs`: `Cell.elevation`
- Research: `environment_climate.md` section 5.2.3
