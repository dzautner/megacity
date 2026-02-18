# DISASTER-025: Earthquake Early Warning System Building

## Priority: T3 (Differentiation)

## Description
Implement an earthquake early warning system as a purchasable building/upgrade that provides 10-60 seconds of warning before earthquake shaking arrives. Warning triggers automatic gas shutoff (prevents post-quake fires), citizen duck-and-cover (reduces casualties), and elevator stops (prevents entrapment).

## Current State
- Earthquakes occur instantly with no warning.
- No preparedness infrastructure.

## Definition of Done
- [ ] Earthquake Early Warning System: $500K building, 1x1 footprint.
- [ ] Warning time: 10-60 seconds (game equivalent: earthquake damage reduced by 20% for secondary effects).
- [ ] Auto gas shutoff: reduces post-quake fire probability by 80%.
- [ ] Citizen alert: reduces casualties by 30%.
- [ ] Elevator safety: prevents entrapment (minor quality-of-life).
- [ ] Only effective if building exists AND has power.
- [ ] UI: earthquake warning banner with countdown.

## Test Plan
- [ ] Unit test: warning system reduces fire probability by 80%.
- [ ] Unit test: casualty reduction of 30%.
- [ ] Integration test: city with EWS has fewer earthquake deaths.
- [ ] Integration test: EWS without power provides no warning.

## Pitfalls
- "10-60 seconds" in real time is essentially instant in game time; translate to damage reduction.
- Power dependency means blackout + earthquake = no warning (realistic but harsh).
- Must coordinate with DISASTER-003 (secondary effects) and DISASTER-004 (preparedness).

## Code References
- `crates/simulation/src/disasters.rs`: earthquake processing
- Research: `environment_climate.md` section 5.1.5
