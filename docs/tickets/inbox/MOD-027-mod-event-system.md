# MOD-027: Mod Event System (Subscribe/Emit)

## Priority: T5 (Stretch)
## Effort: Medium (3-5 days)
## Source: modding_architecture.md -- Event API

## Description
Mods can subscribe to game events (on_building_placed, on_citizen_born, on_disaster, on_budget_cycle) and emit custom events. Event bus shared between Bevy systems and mod runtimes.

## Acceptance Criteria
- [ ] Event registry with subscribe/emit API
- [ ] Built-in events: building_placed, building_demolished, citizen_born, citizen_died, disaster_start, budget_cycle
- [ ] Custom mod events with string names
- [ ] Event data passed as key-value table
- [ ] Events dispatched in deterministic order
