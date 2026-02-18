# INFRA-137: Happiness Formula Weight Tuning and Diminishing Returns
**Priority:** T1
**Complexity:** M (1-3 days)
**Dependencies:** none
**Source:** master_architecture.md, Section 6.4

## Description
Tune happiness formula weights and add diminishing returns / critical thresholds. Currently all factors contribute linearly. Change to: each factor has diminishing returns curve (first unit of service matters most), critical thresholds (below threshold = severe penalty, above = moderate bonus), and configurable weights. Add missing factors: weather happiness, wealth satisfaction. Update happiness more frequently (current 100-tick timer feels laggy).

## Definition of Done
- [ ] Diminishing returns curve for each happiness factor
- [ ] Critical thresholds (e.g., no water = -50% happiness)
- [ ] Weather happiness factor
- [ ] Wealth satisfaction factor
- [ ] Configurable weights (move to config file)
- [ ] Faster happiness update (every 20-50 ticks)
- [ ] Tests pass

## Test Plan
- Unit: First hospital in area provides large happiness boost; third provides small boost
- Unit: No water service = severe happiness penalty
- Integration: Happiness responds to changes within reasonable time

## Pitfalls
- Rebalancing happiness affects immigration, emigration, building upgrades -- cascade
- Update timer too fast = performance; too slow = laggy feel
- Need playtesting for weight balance

## Relevant Code
- `crates/simulation/src/happiness.rs` -- happiness computation
