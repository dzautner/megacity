# CIT-063: Non-Linear Road Degradation (S-Curve)

**Priority:** T2 (Depth)
**Complexity:** Low (0.5 person-weeks)
**Dependencies:** None
**Source:** master_architecture.md Section 1.2

## Description

Replace linear road degradation with S-curve model. Years 0-10: slow degradation (PCI 100->85). Years 10-15: accelerating (PCI 85->50). Years 15-20: rapid failure (PCI 50->10). The "1-to-6 rule": $1 in maintenance saves $6 in rehabilitation. Maintenance at PCI>60 is preventive (cheap, effective). Rehabilitation at PCI<40 costs 6x. Failed roads (PCI<20) need full reconstruction. Current RoadConditionGrid uses linear degradation.

## Definition of Done

- [ ] S-curve degradation function (slow-accelerating-rapid)
- [ ] PCI = f(age, traffic_load, maintenance_history)
- [ ] Preventive maintenance: cheap at PCI>60
- [ ] Rehabilitation: 6x cost at PCI<40
- [ ] Reconstruction: 10x cost at PCI<20
- [ ] Maintenance budget allocation (automatic or manual)
- [ ] Road condition overlay showing PCI gradient
- [ ] "1-to-6 rule" reflected in costs

## Test Plan

- Unit test: PCI at year 5 > 85
- Unit test: PCI at year 15 ~ 50 (without maintenance)
- Unit test: preventive maintenance cost < rehabilitation cost
- Integration test: neglected roads visibly degrade over time

## Pitfalls

- Current RoadConditionGrid uses u8 PCI; S-curve needs continuous time tracking
- Maintenance must be automatic (player shouldn't micro-manage each road)

## Relevant Code

- `crates/simulation/src/road_maintenance.rs` (RoadConditionGrid)
