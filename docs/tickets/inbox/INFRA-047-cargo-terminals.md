# INFRA-047: Cargo Terminals and Port/Rail Freight
**Priority:** T3
**Complexity:** M (1-3 days)
**Dependencies:** INFRA-046
**Source:** transportation_simulation.md, Section 5.3

## Description
Implement cargo terminal buildings that serve as freight origins/destinations. Types: SmallPort (1000 tons/day, 50 truck trips), LargePort (10000 tons/day, 500 truck trips), RailYard (5000 tons/day, 250 truck trips), AirCargo (500 tons/day, 100 truck trips), TruckTerminal (2000 tons/day, 200 truck trips). Conversion: truck_trips = tonnage / avg_payload (~20 tons). Rail connection removes ~50% of truck trips from road network.

## Definition of Done
- [ ] `CargoTerminal` struct with type, capacity, throughput
- [ ] `TerminalType` enum with 5 variants
- [ ] Truck trip generation from terminal throughput
- [ ] Rail connection reduces road truck traffic
- [ ] Terminal placement tool
- [ ] Tests pass

## Test Plan
- Unit: SmallPort generates 50 truck trips/day
- Unit: Rail connection reduces truck trips by 50%

## Pitfalls
- Terminals need road access (must be adjacent to arterial/highway)
- Port must be at water edge; rail yard needs rail connection (not yet implemented)
- Empty truck return trips double the total movements

## Relevant Code
- `crates/simulation/src/outside_connections.rs` -- external freight connections
- `crates/simulation/src/production.rs` -- commodity throughput
