# CIT-076: Outside Connections and Trade Routes

**Priority:** T2 (Depth)
**Complexity:** Medium (2-3 person-weeks)
**Dependencies:** CIT-075 (production chains)
**Source:** master_architecture.md Section 1.8

## Description

Trade connections with the outside world via highway, rail, sea, and air. Each connection type has capacity (trucks/day, containers/day, flights/day). Import costs increase with distance and congestion. Export revenue from local production surplus. Trade balance metric (exports - imports). Outside connections also serve immigration (people arrive via trade routes). Airport passenger capacity limits tourism and immigration.

## Definition of Done

- [ ] Trade connection types with capacity
- [ ] Import cost calculation
- [ ] Export revenue from surplus goods
- [ ] Trade balance metric
- [ ] Connection capacity affects max trade volume
- [ ] Immigration via trade connections
- [ ] Airport passenger capacity
- [ ] Trade stats in economy panel

## Test Plan

- Unit test: import cost increases with distance
- Unit test: export revenue from surplus
- Unit test: capacity limits trade volume
- Integration test: trade connections affect city economy

## Pitfalls

- OutsideConnections already exists; enhance rather than replace

## Relevant Code

- `crates/simulation/src/outside_connections.rs` (OutsideConnections)
- `crates/simulation/src/imports_exports.rs` (TradeConnections)
- `crates/simulation/src/airport.rs`
