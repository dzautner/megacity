# INFRA-120: Grid Battery Storage
**Priority:** T3
**Complexity:** M (1-3 days)
**Dependencies:** INFRA-119
**Source:** infrastructure_engineering.md, Section 7 (Duck Curve)

## Description
Implement grid-scale battery storage to address duck curve and intermittent renewable challenges. Batteries charge during surplus (midday solar) and discharge during evening peak. Instant response time (milliseconds). Capacity in MWh, power in MW. Battery degradation over charge cycles. Cost: $200-400/kWh declining. Battery storage eliminates need for expensive peaker plants.

## Definition of Done
- [ ] Battery storage building with capacity (MWh) and power (MW)
- [ ] Charge during surplus, discharge during deficit
- [ ] Battery degradation over cycles
- [ ] Reduces need for peaker plants
- [ ] Battery status in power info panel (charge level, cycles)
- [ ] Tests pass

## Test Plan
- Unit: Battery charges when solar surplus, discharges at evening peak
- Unit: Battery capacity degrades over 10-year lifespan
- Integration: Solar + battery eliminates evening brownouts

## Pitfalls
- Battery sizing: too small = still need peakers; too large = wasted money
- Degradation rate affects lifecycle economics
- Fire risk for large lithium battery installations (NIMBY)

## Relevant Code
- `crates/simulation/src/utilities.rs` -- power dispatch
- `crates/simulation/src/buildings.rs` -- battery building type
