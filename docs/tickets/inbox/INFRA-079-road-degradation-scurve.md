# INFRA-079: S-Curve Road Degradation Model
**Priority:** T2
**Complexity:** S (hours)
**Dependencies:** none
**Source:** master_architecture.md, Section 5.1; infrastructure_engineering.md

## Description
Replace linear road degradation with S-curve model: roads degrade slowly when new, accelerate in middle life, then plateau near failure. Use logistic function: `condition = 1.0 / (1.0 + exp(k * (age - midpoint)))`. Heavy truck traffic accelerates degradation (ESAL factor: 4th power of axle weight ratio). Preventive maintenance (resurfacing at 60-70% condition) is much cheaper than reconstruction (at 20% condition).

## Definition of Done
- [ ] S-curve degradation function replacing linear decay
- [ ] Heavy vehicle traffic multiplier (ESAL factor)
- [ ] Preventive maintenance option (cheaper if condition > 60%)
- [ ] Reconstruction option (expensive, required below 20%)
- [ ] Road condition affects vehicle speed
- [ ] Tests pass

## Test Plan
- Unit: New road degrades slowly in first 25% of life
- Unit: Heavy truck traffic accelerates degradation
- Unit: Preventive maintenance cheaper than reconstruction

## Pitfalls
- S-curve parameters need tuning for game-year time scale
- Truck ESAL: a 40-ton truck does 10,000x more damage than a 1-ton car (4th power law)

## Relevant Code
- `crates/simulation/src/road_maintenance.rs` -- current degradation model
