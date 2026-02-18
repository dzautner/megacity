# DISASTER-014: Disaster Recovery Framework (Emergency, Assessment, Repair, Rebuild)

## Priority: T2 (Depth)

## Description
Implement a common disaster recovery process for all disaster types. Currently destroyed buildings are simply despawned. The research doc specifies 5 recovery phases (Emergency, Assessment, Repair, Rebuild, Recovered) with costs, durations, and dependencies on city resources.

## Current State
- Destroyed buildings are despawned immediately.
- No repair concept for damaged (non-destroyed) buildings.
- No recovery cost tracking.
- No displaced citizen tracking.
- No emergency/federal aid system.

## Definition of Done
- [ ] `DisasterRecovery` resource tracking recovery state per disaster event.
- [ ] Recovery phases: Emergency (1-3 days), Assessment (1-2 days), Repair (weeks), Rebuild (months).
- [ ] Damaged buildings have `DamageState` component (Moderate, Severe) requiring repair.
- [ ] Repair cost: Moderate = 10-30% building value, Severe = 50-80%.
- [ ] Rebuild cost: 100% of building value for destroyed structures.
- [ ] Recovery speed depends on: budget, emergency services, construction workforce.
- [ ] Displaced citizens: occupy shelters or leave city.
- [ ] Insurance: covers 80% of damage if purchased (2% property value/year policy).
- [ ] Federal aid: available when damage > 10% of city budget, covers 75%.
- [ ] City disaster fund: player pre-funds emergency reserve.

## Test Plan
- [ ] Unit test: recovery progresses through phases in correct order.
- [ ] Unit test: repair cost matches damage state percentage.
- [ ] Integration test: damaged buildings are repaired over time if budget allows.
- [ ] Integration test: federal aid activates when damage exceeds threshold.

## Pitfalls
- Damaged buildings must function at reduced capacity during repair (not instantly broken).
- Insurance as a pre-purchase decision creates strategic depth but may be confusing.
- Recovery timeline must feel appropriate (not too fast or too slow).

## Code References
- `crates/simulation/src/disasters.rs`: `ActiveDisaster`
- Research: `environment_climate.md` section 5.7
