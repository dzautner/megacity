# BLDG-006: Construction Material Cost
**Priority:** T1
**Complexity:** S
**Dependencies:** none
**Source:** master_architecture.md, section M2

## Description
Deduct construction cost from city treasury when buildings spawn. Currently buildings appear for free. Add a per-building construction cost based on zone type and level that is deducted from treasury at construction start.

- Base construction costs: R-1 L1=$500, R-4 L1=$5000, C-1 L1=$1000, C-4 L1=$8000, I-1 L1=$3000, O L1=$4000
- Cost scales with level: L2=3x, L3=8x, L4=20x, L5=50x base
- Deducted when building_spawner creates entity (before UnderConstruction begins)
- If treasury < construction cost, skip spawning that building
- Construction cost also applies to upgrades (level-up costs money)
- Display cost in tooltip when hovering over zone

## Definition of Done
- [ ] Construction cost defined per zone/level
- [ ] Treasury deducted at building spawn
- [ ] Insufficient funds prevents building spawn
- [ ] Upgrade costs deducted from treasury
- [ ] Cost visible in UI

## Test Plan
- Unit: construction_cost(ResidentialLow, 1) > 0
- Integration: Empty treasury prevents building spawns
- Integration: After building spawns, treasury reduced by expected amount

## Pitfalls
- Early game balance: starting treasury must cover initial buildings
- Cost must be balanced against tax income (positive ROI within reasonable timeframe)
- Upgrade cost should not prevent upgrades for players already struggling financially

## Relevant Code
- `crates/simulation/src/buildings.rs:building_spawner` -- deduct cost before spawn
- `crates/simulation/src/building_upgrade.rs` -- deduct upgrade cost
- `crates/simulation/src/economy.rs:CityBudget` -- treasury modification
