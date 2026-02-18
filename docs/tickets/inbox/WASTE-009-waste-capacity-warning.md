# WASTE-009: Landfill Capacity Warning and Emergency Alerts

## Priority: T1 (Core)

## Description
Implement warning events when landfill capacity is running low. Alert the player at 75%, 50%, 25%, and 10% remaining capacity with increasing urgency. At 0%, waste collection stops and uncollected waste accumulates everywhere.

## Current State
- No landfill capacity tracking.
- No warning system for any utility capacity.

## Definition of Done
- [ ] `WasteCapacityWarning` event triggered at capacity thresholds.
- [ ] Warning tiers: 25% remaining (5 years notice), 10% (2 years), 5% (critical), 0% (emergency).
- [ ] UI notification for each warning tier with suggested actions.
- [ ] At 0% capacity: all waste becomes uncollected, health crisis begins.
- [ ] Suggested actions: expand landfill, build recycling center, build WTE, enable composting.
- [ ] Warning event visible in event log and as overlay icon.

## Test Plan
- [ ] Unit test: warning triggered at each threshold.
- [ ] Unit test: 0% capacity stops waste collection.
- [ ] Integration test: landfill filling up triggers progressive warnings.
- [ ] Integration test: building additional waste infrastructure resolves warning.

## Pitfalls
- Warning timing depends on fill rate, which changes as city grows.
- Years remaining estimate may be inaccurate if growth rate changes.
- Emergency at 0% must be severe enough to motivate player action.

## Code References
- Research: `environment_climate.md` sections 6.3, 8.4
