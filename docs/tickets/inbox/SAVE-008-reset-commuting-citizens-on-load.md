# SAVE-008: Reset Commuting Citizens to AtHome on Load

## Priority: T1 (Short-Term Fix)
## Effort: Small (0.5 day)
## Source: save_system_architecture.md -- Issue 2: PathCache and Velocity Not Serialized

## Description
Commuting citizens load with empty PathCache and zero Velocity, causing them to freeze. Reset all commuting states to AtHome on load so the movement system naturally re-dispatches them.

## Acceptance Criteria
- [ ] Citizens with commuting states are set to AtHome on load
- [ ] Position set to home coordinates
- [ ] Movement system re-dispatches them naturally
- [ ] No visible teleportation or freezing
