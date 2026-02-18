# POWER-018: Power Plant Maintenance Schedules and Forced Outages

## Priority: T3 (Differentiation)

## Description
Implement power plant maintenance schedules (planned downtime) and random forced outages. Each plant type has a maintenance interval and outage probability. Neglecting maintenance increases forced outage risk.

## Current State
- No maintenance concept for any building.
- No forced outage probability.
- Power plants (when implemented) would run 24/7 without interruption.

## Definition of Done
- [ ] Planned maintenance: each plant requires N game-days of maintenance per year.
- [ ] During maintenance: plant output = 0.
- [ ] Player can schedule maintenance timing (prefer shoulder seasons).
- [ ] Forced outage probability: coal=5%/month, gas=3%, nuclear=1%, solar=2% (inverter failure), wind=4% (mechanical).
- [ ] Outage duration: coal=3-7 days, gas=1-3 days, nuclear=7-30 days.
- [ ] Deferred maintenance: outage probability increases 2x per missed interval.
- [ ] Multiple simultaneous outages can cause reserve margin crisis.

## Test Plan
- [ ] Unit test: plant at 0 output during maintenance window.
- [ ] Unit test: deferred maintenance doubles outage probability.
- [ ] Integration test: random outage reduces total supply temporarily.
- [ ] Integration test: multiple outages trigger reserve margin warning.

## Pitfalls
- Player may not understand maintenance requirements without clear UI.
- Nuclear outage duration (30 days) is very long; may need scaling.
- Maintenance scheduling UI needs calendar-like interface.

## Code References
- Research: `environment_climate.md` section 3.3
