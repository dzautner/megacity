# INFRA-070: Construction Material Cost for Buildings
**Priority:** T1
**Complexity:** S (hours)
**Dependencies:** none
**Source:** master_architecture.md, M2

## Description
Add construction cost deducted from budget when buildings spawn. Cost varies by zone type and building level. Residential L1: $5K, L3: $50K, L5: $200K. Commercial: 1.5x residential. Industrial: 0.8x residential. Buildings cannot spawn if budget cannot cover construction cost. This adds economic constraint to growth.

## Definition of Done
- [ ] Construction cost per building type and level
- [ ] Cost deducted from budget at building spawn
- [ ] Building spawn blocked when budget insufficient
- [ ] Construction cost shown in building info panel
- [ ] Tests pass

## Test Plan
- Unit: Building spawn deducts correct cost from treasury
- Unit: Building does not spawn when treasury < construction cost
- Integration: Rapid growth drains treasury, slowing further growth

## Pitfalls
- Currently buildings spawn for free; adding cost will slow early game significantly
- May need to tune costs for game balance
- Building upgrades (L1->L2) should also have upgrade cost

## Relevant Code
- `crates/simulation/src/buildings.rs` -- building_spawner
- `crates/simulation/src/economy.rs` -- treasury
