# SERV-001: Service Capacity Limits
**Priority:** T1
**Complexity:** M
**Dependencies:** none
**Source:** master_architecture.md, section M2; cities_skylines_analysis.md, section 8

## Description
Implement capacity limits for service buildings. Currently services provide unlimited coverage within radius. A hospital should have finite beds, a school finite seats, a fire station finite trucks.

- Hospital: 100-500 beds depending on type. Overcrowded hospital = reduced health coverage
- Elementary school: 200-500 students. Full school = children uneducated
- High school: 500-1000 students
- University: 1000-5000 students
- Fire station: 3-6 trucks. Busy trucks = slower response time
- Police station: 10-30 officers. High crime + low officers = ineffective policing
- Each service: capacity field, current_usage field, utilization = usage/capacity

## Definition of Done
- [ ] All service buildings have capacity limits
- [ ] Utilization tracked (current_usage / capacity)
- [ ] Over-capacity reduces effectiveness
- [ ] Utilization visible in service building info
- [ ] Status icons for over-capacity buildings

## Test Plan
- Unit: Hospital at 100% capacity provides reduced health coverage
- Integration: Build one hospital for 50K population, verify overcrowding
- Integration: Add second hospital, verify utilization drops

## Pitfalls
- Current coverage-radius model doesn't track individual service consumers
- Need to assign citizens to nearest service building with capacity
- SpatialIndex on DestinationCache could help with nearest-service lookup
- Over-capacity must degrade gracefully, not binary on/off

## Relevant Code
- `crates/simulation/src/services.rs:ServiceBuilding` -- add capacity, current_usage fields
- `crates/simulation/src/happiness.rs:ServiceCoverageGrid` -- check capacity before marking covered
- `crates/rendering/src/status_icons.rs` -- overcrowding icon
