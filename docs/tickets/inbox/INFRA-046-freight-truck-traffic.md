# INFRA-046: Freight and Truck Traffic Simulation
**Priority:** T2
**Complexity:** L (3-5 days)
**Dependencies:** INFRA-030, INFRA-032
**Source:** transportation_simulation.md, Section 5.1-5.2

## Description
Model freight traffic on road network. Industrial zones generate truck trips (4-15 trips/day/1000sqm). Trucks have Passenger Car Equivalents (PCE): semi-trailer=2.0, single-unit=1.5. Convert truck traffic to PCE for V/C calculations. Temporal pattern: trucks peak early morning (1.5x), lowest at night (0.4x). Truck route restrictions: heavy trucks banned from `RoadType::Local` and `Path`. Freight OD matrix based on supply chain: Industrial->Warehouse 30%, Industrial->Commercial 25%, etc.

## Definition of Done
- [ ] Truck trip generation per industrial/commercial building
- [ ] PCE conversion for V/C calculation
- [ ] Truck route restrictions by road type
- [ ] Time-of-day freight traffic variation
- [ ] Freight OD distribution (industrial->warehouse->commercial)
- [ ] Freight traffic visible in traffic overlay
- [ ] Tests pass

## Test Plan
- Unit: 10% trucks with PCE 2.0 = 1.1x effective volume
- Unit: Truck routing avoids Local roads
- Integration: Industrial zones generate visible truck traffic on arterials

## Pitfalls
- Truck routing failures when no arterial connects to industrial zone
- PCE on hills is higher (5.0 at 4% grade for semi-trailers)
- Freight traffic without explicit trip generation may look wrong; use realistic rates

## Relevant Code
- `crates/simulation/src/traffic.rs` -- traffic volume tracking with PCE
- `crates/simulation/src/movement.rs` -- freight movement entities
- `crates/simulation/src/production.rs` -- commodity flow for OD matrix
