# INFRA-057: Service Building Capacity Limits
**Priority:** T1
**Complexity:** M (1-3 days)
**Dependencies:** none
**Source:** master_architecture.md, M2; Section 6.5

## Description
Add capacity limits to service buildings: hospital beds, school seats, fire station truck count, police patrol capacity. When demand exceeds capacity, service quality degrades (not a binary on/off). Currently services use coverage-radius only with no capacity check. A hospital with 10 beds serving 50,000 people should have degraded healthcare quality.

## Definition of Done
- [ ] `ServiceCapacity` component on service buildings
- [ ] Capacity defined per service building type
- [ ] Service quality = min(1.0, capacity / demand) within coverage radius
- [ ] Overcrowded services show warning icons
- [ ] Quality affects happiness contribution from that service
- [ ] Tests pass

## Test Plan
- Unit: Hospital with 100 beds serving 50 patients = 100% quality
- Unit: Hospital with 100 beds serving 200 patients = 50% quality
- Integration: Building more hospitals improves healthcare when overcrowded

## Pitfalls
- Current `ServiceCoverageGrid` is bitflag-based; need quality-weighted version
- Multiple service buildings with overlapping coverage should pool capacity
- School capacity: separate by level (elementary, high school, university)

## Relevant Code
- `crates/simulation/src/services.rs` -- service coverage system
- `crates/simulation/src/happiness.rs` -- service contribution to happiness
- `crates/rendering/src/status_icons.rs` -- overcrowding icon
