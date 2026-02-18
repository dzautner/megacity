# SVC-012: City Hall Administration Efficiency

**Priority:** T3 (Differentiation)
**Complexity:** Low-Medium (1 person-week)
**Dependencies:** None
**Source:** historical_demographics_services.md Section 5.1

## Description

City Hall provides city-wide administration bonus. Target: 100-200 admin staff per 100K population. Understaffed city hall: slower permit processing (-25% building construction speed), worse tax collection (-10% revenue), more corruption (+corruption metric). City hall location matters: central location boosts civic pride (+5 happiness city-wide). CityHall must be built early; upgrade as city grows. Three tiers: Small (pop < 25K), Medium (25-100K), Large (100K+).

## Definition of Done

- [ ] CityHall tiers (Small/Medium/Large) with staff requirements
- [ ] Administration efficiency = staff_assigned / staff_required
- [ ] Low efficiency: -25% construction speed, -10% tax revenue
- [ ] High efficiency: +5% construction speed, +5% tax revenue
- [ ] Central location bonus (+5 happiness city-wide)
- [ ] Civic pride metric
- [ ] Required building for governance features

## Test Plan

- Unit test: understaffed city hall reduces construction speed
- Unit test: centrally located city hall provides happiness bonus
- Integration test: growing city without upgraded city hall shows declining efficiency

## Pitfalls

- "Central location" needs definition (distance from grid center? average citizen distance?)
- City Hall already exists in ServiceType; needs functional implementation

## Relevant Code

- `crates/simulation/src/services.rs` (ServiceType::CityHall)
