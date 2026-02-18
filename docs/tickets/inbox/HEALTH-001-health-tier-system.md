# HEALTH-001: Healthcare Multi-Tier System

**Priority:** T2 (Depth)
**Complexity:** Medium (2 person-weeks)
**Dependencies:** SVC-002 (capacity limits)
**Source:** historical_demographics_services.md Section 3.3

## Description

Differentiate healthcare tiers functionally. MedicalClinic: 20 beds, 10 staff, walk-in care, handles minor illness. Hospital: 200 beds, 100+ staff, emergency + surgical, handles all conditions. MedicalCenter: 500 beds, 300+ staff, research capability, handles rare diseases, training bonus for education. Bed occupancy target: 65-85%. Over 85% = declining quality. Under 50% = wasteful spending. Each tier handles different patient acuity levels.

## Definition of Done

- [ ] MedicalClinic: 20 beds, minor conditions, preventive care
- [ ] Hospital: 200 beds, emergency/surgical, all conditions
- [ ] MedicalCenter: 500 beds, research, rare diseases, training
- [ ] Bed occupancy tracking per facility
- [ ] Optimal occupancy: 65-85% (quality = 1.0)
- [ ] Over-occupancy: quality degrades linearly (0.5 at 150% capacity)
- [ ] Ambulance dispatch for emergencies (SVC-003 integration)
- [ ] Health coverage from road-network distance, not Euclidean

## Test Plan

- Unit test: clinic handles minor illness
- Unit test: hospital handles emergencies
- Unit test: over-capacity hospital has degraded quality
- Integration test: building medical facilities improves health grid

## Pitfalls

- Patient routing: minor to clinic, serious to hospital; need acuity system
- Healthcare demand varies by age (5x for seniors); must model age distribution

## Relevant Code

- `crates/simulation/src/health.rs` (HealthGrid, update_health_grid)
- `crates/simulation/src/services.rs` (MedicalClinic, Hospital, MedicalCenter)
