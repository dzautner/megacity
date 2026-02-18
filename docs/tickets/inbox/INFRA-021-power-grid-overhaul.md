# INFRA-021: Power Grid with Generation Mix and Demand/Supply Balance
**Priority:** T1
**Complexity:** L (3-5 days)
**Dependencies:** none
**Source:** underground_infrastructure.md, Power Grid section; master_architecture.md, M2

## Description
Overhaul the power system from simple BFS coverage to a proper generation/distribution model. Power plants produce MW. Substations distribute power. Transmission lines connect plants to substations. Each building has power demand (residential ~2kW, commercial ~5kW, industrial ~20kW). When demand > supply: brownouts (random building power loss), then blackouts (area-wide). Support generation types: coal, natural gas, solar, wind, nuclear with different costs, capacities, and pollution.

## Definition of Done
- [ ] Power demand per building type
- [ ] Power plant entities with capacity (MW) and fuel type
- [ ] Substation distribution with capacity limits
- [ ] Brownout/blackout when demand > supply
- [ ] Multiple generation types with distinct tradeoffs
- [ ] Power demand/supply overlay
- [ ] Tests pass

## Test Plan
- Unit: City with 1000 homes needs ~2MW; 1MW plant causes brownouts
- Unit: Coal plant produces pollution, solar does not
- Integration: Building a new power plant resolves brownouts

## Pitfalls
- Current power BFS in `utilities.rs` needs careful migration
- Power demand varies by time of day (peak afternoon/evening)
- Solar/wind are intermittent; need battery storage or backup plants

## Relevant Code
- `crates/simulation/src/utilities.rs` -- current power BFS
- `crates/simulation/src/economy.rs` -- power plant operating costs
- `crates/simulation/src/pollution.rs` -- power plant emissions
