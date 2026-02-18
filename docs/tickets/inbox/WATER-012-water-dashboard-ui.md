# WATER-012: Water Supply Dashboard UI Panel

## Priority: T1 (Core)

## Description
Create a water supply dashboard showing total demand, total supply, service coverage, groundwater level, reservoir level, and treatment status. This is the player's interface for managing water infrastructure.

## Current State
- No water-specific dashboard.
- Groundwater level shown in stats but not prominently.

## Definition of Done
- [ ] Dashboard showing: total demand (MGD), total supply (MGD), surplus/deficit.
- [ ] Source breakdown: wells, surface intake, reservoir, desalination contributions.
- [ ] Groundwater level indicator (with depletion warning).
- [ ] Reservoir level (if exists): % full, days of storage.
- [ ] Service coverage: % of buildings with water service.
- [ ] Water quality: treatment level and output quality.
- [ ] Sewage treatment: % of wastewater treated, treatment level.
- [ ] Monthly water budget: treatment costs, revenue from water rates.

## Test Plan
- [ ] UI test: demand and supply correctly displayed.
- [ ] UI test: groundwater warning at low levels.
- [ ] UI test: service coverage updates when infrastructure changes.

## Pitfalls
- Requires WATER-001 and WATER-002 for meaningful data.
- Multiple source types need visual distinction.
- Monthly budget integration with economy system.

## Code References
- `crates/ui/src/info_panel.rs`: existing panel system
- `crates/simulation/src/groundwater.rs`: `GroundwaterStats`
