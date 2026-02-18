# POWER-016: Blackout and Rolling Blackout System

## Priority: T1 (Core)

## Description
Implement blackout mechanics when power demand exceeds supply. Buildings without power lose function, citizens lose happiness, and critical facilities (hospitals, fire stations) may cause casualties. Rolling blackouts rotate affected areas to distribute the burden.

## Current State
- No blackout system.
- No consequence for power deficit.
- No load priority system.

## Definition of Done
- [ ] Blackout triggers when `EnergyGrid.reserve_margin < 0` (demand > supply).
- [ ] Load priority tiers: Critical (hospitals, fire, police), Essential (water, sewer, transit), Standard (residential, commercial), Deferrable (non-essential, EV charging).
- [ ] Shed load in reverse priority order until supply matches demand.
- [ ] Rolling blackout: rotate affected Standard cells every 4 ticks.
- [ ] `has_power` flag on each building, set by dispatch system.
- [ ] No-power effects: happiness -10, no heating/cooling, no industrial production, buildings at 0% function.
- [ ] Hospital without power: 5% patient mortality per game-day without power.
- [ ] Duration tracking: extended blackouts (>3 game-days) trigger citizen exodus.

## Test Plan
- [ ] Unit test: Critical facilities shed last.
- [ ] Unit test: rolling blackout rotates affected areas each cycle.
- [ ] Integration test: demand exceeding supply triggers blackout.
- [ ] Integration test: building more generation resolves blackout.

## Pitfalls
- Rolling blackout rotation must be fair (not always the same area).
- Hospital mortality is a severe consequence; players need clear warning.
- Must integrate with power line connectivity (POWER-011).

## Code References
- Research: `environment_climate.md` sections 3.4, 3.5.4
