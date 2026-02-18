# POWER-019: Energy Dashboard UI Panel

## Priority: T1 (Core)

## Description
Create an energy dashboard UI panel showing total demand, total supply, reserve margin, generation mix (pie chart), electricity price, and blackout status. This is the primary player interface for managing the power system.

## Current State
- No energy-related UI panel.
- No generation mix visualization.
- No demand/supply display.

## Definition of Done
- [ ] Dashboard panel showing: total demand (MW), total supply (MW), reserve margin (%).
- [ ] Generation mix: bar or pie showing MW from each plant type (coal, gas, solar, wind, etc.).
- [ ] Current electricity price ($/kWh).
- [ ] Blackout status indicator (green/yellow/red for healthy/warning/blackout).
- [ ] History graph: demand and supply over last 24 game-hours.
- [ ] Expandable section showing each generator's status (output, fuel cost, maintenance).
- [ ] Accessible from toolbar or info panel.

## Test Plan
- [ ] UI test: dashboard displays correct demand and supply values.
- [ ] UI test: blackout indicator turns red when demand > supply.
- [ ] UI test: generation mix updates when plants are built/retired.

## Pitfalls
- Complex UI layout; should be readable at a glance with details on drill-down.
- History graph requires storing 24-hour history (ring buffer).
- Must update in real-time as game ticks advance.

## Code References
- `crates/ui/src/info_panel.rs`: existing panel system
- `crates/ui/src/toolbar.rs`: toolbar buttons
