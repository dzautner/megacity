# POWER-012: Demand Response Programs

## Priority: T3 (Differentiation)

## Description
Implement demand response programs that reduce peak electricity demand through policy rather than building more generators. Programs include smart thermostats, industrial load shifting, EV managed charging, peak pricing signals, and interruptible service contracts.

## Current State
- No demand response concept.
- No peak demand management.

## Definition of Done
- [ ] Smart thermostat program: -8% peak demand, $1M cost, minor comfort impact.
- [ ] Industrial load shifting: -12% peak, $500K, shifts production timing.
- [ ] EV managed charging: -5% peak, $300K, convenience impact.
- [ ] Peak pricing signals: -10% peak, $0 cost, higher bills for citizens.
- [ ] Interruptible service: -15% peak, $2M rebates, occasional outages.
- [ ] Critical peak rebates: -7% peak, $1M rebates, behavior change.
- [ ] Each program toggleable as a policy in the policy panel.
- [ ] Programs reduce `EnergyGrid.peak_demand_mwh`.

## Test Plan
- [ ] Unit test: enabling smart thermostat reduces peak demand by 8%.
- [ ] Unit test: multiple programs stack additively.
- [ ] Integration test: demand response prevents blackout during peak.

## Pitfalls
- Multiple programs stacking could make peak demand trivially easy to manage.
- Citizen impact (comfort, convenience) needs to affect happiness.
- Some programs require prerequisites (EV charging requires EVs in the city).

## Code References
- Research: `environment_climate.md` section 3.5.3
