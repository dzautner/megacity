# POWER-008: Battery Energy Storage System

## Priority: T2 (Depth)

## Description
Implement battery storage facilities that store excess electricity during off-peak/high-renewable periods and discharge during peak demand or renewable lulls. Essential for making solar and wind viable at scale.

## Current State
- No energy storage exists.
- No time-shifting of electricity supply.

## Definition of Done
- [ ] Battery storage: capacity in MWh, charge/discharge rate in MW.
- [ ] Small battery: 10 MWh, 5 MW rate, $5M.
- [ ] Large battery: 100 MWh, 50 MW rate, $40M.
- [ ] Charge when supply > demand; discharge when demand > supply.
- [ ] Round-trip efficiency: 85% (15% energy loss).
- [ ] State of charge (SOC) tracked: 0-100%.
- [ ] Reserve threshold: always keep 20% stored for emergencies.
- [ ] Hospital/critical facilities have dedicated backup batteries.
- [ ] 2x2 building footprint.

## Test Plan
- [ ] Unit test: battery charges at rate limit when excess supply.
- [ ] Unit test: round-trip efficiency loses 15% of stored energy.
- [ ] Unit test: battery stops discharging at 20% SOC reserve.
- [ ] Integration test: solar + battery provides 24-hour power.
- [ ] Integration test: battery prevents blackout during evening peak.

## Pitfalls
- Dispatch logic must decide when to charge vs discharge (POWER-009 dependency).
- Multiple batteries must coordinate (don't all discharge simultaneously).
- Battery degradation over time is realistic but may be too complex for first pass.

## Code References
- Research: `environment_climate.md` sections 3.3, 3.4
