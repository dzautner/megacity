# INFR-005: Road S-Curve Degradation Model
**Priority:** T1
**Complexity:** S
**Dependencies:** none
**Source:** master_architecture.md, section 5.1

## Description
Replace linear road degradation with S-curve model. Roads degrade slowly initially, then rapidly after reaching middle life, then level off at "failed" state. More realistic than current linear model.

- S-curve: condition = 100 / (1 + exp(k * (age - midpoint)))
- New road: condition ~100, gradual decline for first 50% of life
- Mid-life: rapid decline (this is when maintenance matters most)
- End of life: condition near 0, road needs full reconstruction
- Heavy traffic accelerates degradation (truck factor)
- Road maintenance (plowing, patching) extends life but doesn't reset

## Definition of Done
- [ ] S-curve degradation formula
- [ ] Degradation rate affected by traffic volume
- [ ] Road condition affects vehicle speed
- [ ] Maintenance extends road life
- [ ] Road condition overlay

## Test Plan
- Unit: New road has condition ~100
- Unit: Old road with heavy traffic has condition < 30

## Pitfalls
- road_maintenance.rs already has degradation -- replace formula
- S-curve parameters need tuning (midpoint, steepness)
- Road reconstruction cost must be defined

## Relevant Code
- `crates/simulation/src/road_maintenance.rs` -- S-curve degradation
- `crates/simulation/src/traffic.rs` -- traffic volume as degradation input
