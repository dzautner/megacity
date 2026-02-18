# INFRA-048: Last-Mile Delivery Congestion in Commercial Zones
**Priority:** T3
**Complexity:** S (hours)
**Dependencies:** INFRA-046
**Source:** transportation_simulation.md, Section 5.4

## Description
Model delivery vehicle congestion in commercial zones. During business hours (8-18), commercial streets get 10-20% capacity reduction from double-parking delivery vehicles. Mitigations: loading zones (50% reduction in penalty), off-peak delivery policy (30% reduction). `delivery_congestion_factor()` modifies road capacity for commercial zone road segments.

## Definition of Done
- [ ] `delivery_congestion_factor()` applied to commercial zone roads
- [ ] Capacity reduction during business hours
- [ ] Loading zone building/policy mitigates impact
- [ ] Off-peak delivery policy mitigates impact
- [ ] Tests pass

## Test Plan
- Unit: CommercialHigh zone road loses 20% capacity during business hours
- Unit: Loading zones reduce penalty to 10%

## Pitfalls
- Time-of-day dependency requires `time_of_day.rs` integration
- Loading zones as zone-level policy vs building-level placement

## Relevant Code
- `crates/simulation/src/traffic.rs` -- capacity modification
- `crates/simulation/src/time_of_day.rs` -- business hours check
- `crates/simulation/src/policies.rs` -- off-peak delivery policy
