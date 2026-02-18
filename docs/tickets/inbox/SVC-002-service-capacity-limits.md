# SVC-002: Service Building Capacity Limits

**Priority:** T2 (Depth)
**Complexity:** Medium (2 person-weeks)
**Dependencies:** SVC-001 (hybrid coverage)
**Source:** historical_demographics_services.md Sections 3.1-3.7

## Description

Add capacity tracking to each service building. Hospitals: bed count (50/200/500 by tier). Schools: student capacity (300/1500/5000). Fire stations: truck count (2/5/10). Police stations: officer count (20/50/200). When demand exceeds capacity, service quality degrades. Staffing requirements: buildings need appropriately educated workers from the labor pool. Unstaffed buildings provide no service.

## Definition of Done

- [ ] `ServiceCapacity` component with max_capacity, current_usage, staff_required, staff_assigned
- [ ] Capacity values per ServiceType tier (from research doc tables)
- [ ] Hospital beds tracked (50/200/500 for Clinic/Hospital/MedicalCenter)
- [ ] School student counts (300/1500/5000 for Elementary/HighSchool/University)
- [ ] Fire trucks (2/5/10 for FireHouse/FireStation/FireHQ)
- [ ] Police officers (10/30/100 for Kiosk/Station/HQ)
- [ ] Over-capacity penalty: quality *= capacity / demand when demand > capacity
- [ ] Staffing: unstaffed building provides 0 service
- [ ] Staff drawn from employed citizen pool

## Test Plan

- Unit test: hospital at 100% capacity provides full quality
- Unit test: hospital at 200% demand provides 50% quality
- Unit test: unstaffed school provides 0 education coverage
- Integration test: building more hospitals when beds full improves health coverage

## Pitfalls

- Staff assignment interacts with employment system; don't create circular dependency
- Capacity per building must be configurable for future data-driven design

## Relevant Code

- `crates/simulation/src/services.rs` (ServiceBuilding)
- `crates/simulation/src/education_jobs.rs` (job assignment)
