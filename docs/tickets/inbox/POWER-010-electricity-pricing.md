# POWER-010: Time-of-Use Electricity Pricing and Revenue

## Priority: T2 (Depth)

## Description
Implement time-of-use electricity pricing that affects city revenue and citizen costs. Price varies by time of day and scarcity (reserve margin). This connects energy infrastructure investment to the budget system.

## Current State
- No electricity pricing exists.
- No energy revenue or cost tracking.

## Definition of Done
- [ ] Base rate: $0.12/kWh (adjustable by player).
- [ ] Time-of-use multipliers: off-peak(22-06)=0.6, mid-peak(06-14)=1.0, on-peak(14-22)=1.5.
- [ ] Scarcity multiplier: reserve>20%=1.0, 10-20%=1.2, 5-10%=1.5, 0-5%=2.0, deficit=3.0.
- [ ] `EnergyEconomics` resource tracking revenue (residential, commercial, industrial) and costs (fuel, maintenance, capital).
- [ ] Net energy income = total_revenue - total_costs.
- [ ] Energy costs affect citizen happiness and disposable income.
- [ ] High energy costs deter immigration.

## Test Plan
- [ ] Unit test: off-peak price = base * 0.6.
- [ ] Unit test: scarcity multiplier at 3% reserve = 2.0.
- [ ] Integration test: city with cheap power has lower citizen costs.
- [ ] Integration test: energy revenue appears in city budget.

## Pitfalls
- Player-adjustable base rate needs a UI slider in the budget panel.
- Energy costs must integrate with existing citizen happiness formula.
- Must not make energy pricing so punishing that players can never afford it.

## Code References
- `crates/simulation/src/economy.rs`: budget integration
- `crates/simulation/src/time_of_day.rs`: `GameClock`
- Research: `environment_climate.md` sections 3.5.1-3.5.2
