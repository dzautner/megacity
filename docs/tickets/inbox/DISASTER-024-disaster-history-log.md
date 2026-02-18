# DISASTER-024: Disaster History Log and Statistics

## Priority: T2 (Depth)

## Description
Track all past disasters in a history log with type, date, severity, damage cost, casualties, and recovery status. Display in a disaster history panel. Used for insurance calculations, disaster fund recommendations, and risk analysis.

## Current State
- `DisasterHistory` exists with a Vec of (DisasterType, u32 tick) but no detailed tracking.
- No damage cost or casualty tracking.
- No history UI panel.

## Definition of Done
- [ ] `DisasterRecord` struct: type, game_day, magnitude/EF/MMI, damage_cost, casualties, displaced, affected_cells_count, recovery_duration.
- [ ] All disasters create a record on completion.
- [ ] History UI panel: sortable table of past disasters.
- [ ] Statistics: total damage cost (all-time), worst disaster, average frequency, most common type.
- [ ] Annual disaster report: summary of year's events.
- [ ] Serialize disaster history in save file.
- [ ] Used by insurance system (DISASTER-017) for premium calculation.

## Test Plan
- [ ] Unit test: disaster creates correct history record.
- [ ] Unit test: statistics calculate correctly from history.
- [ ] Integration test: disaster history persists through save/load.
- [ ] UI test: history panel displays all past disasters.

## Pitfalls
- History can grow large over long game sessions; may need capping.
- Damage cost calculation must be accurate (depends on DISASTER-014 recovery framework).
- Casualty tracking requires integration with citizen death system.

## Code References
- `crates/simulation/src/disasters.rs`: `DisasterHistory`
- `crates/save/src/serialization.rs`: save/load
