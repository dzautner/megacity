# INFRA-045: Population-Based Transit Mode Unlocks
**Priority:** T2
**Complexity:** S (hours)
**Dependencies:** INFRA-037
**Source:** transportation_simulation.md, Section 4.2

## Description
Tier transit mode unlocks by population: <10K buses only, 10K-50K unlock BRT (dedicated bus lanes), 50K-150K unlock light rail/tram, 150K+ unlock metro/subway. This follows real-world transit evolution patterns and creates meaningful upgrade decisions. Integrate with existing `unlocks.rs` milestone system.

## Definition of Done
- [ ] Transit mode unlocks added to milestone system
- [ ] Bus available from start
- [ ] BRT unlocks at 10K population
- [ ] Light rail unlocks at 50K
- [ ] Metro unlocks at 150K
- [ ] UI shows locked/unlocked transit options
- [ ] Tests pass

## Test Plan
- Unit: New city has bus only; metro greyed out
- Unit: At 150K pop, metro becomes available

## Pitfalls
- Player may need metro infrastructure before hitting population; consider early unlock via policy
- Unlocks should be clearly communicated (notification/advisor)

## Relevant Code
- `crates/simulation/src/unlocks.rs` -- milestone unlock system
- `crates/ui/src/toolbar.rs` -- transit tool availability
