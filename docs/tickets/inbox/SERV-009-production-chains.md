# SERV-009: Production Chain Commodity System
**Priority:** T3
**Complexity:** XL
**Dependencies:** SERV-008, TRAF-004
**Source:** cities_skylines_analysis.md, section 14.7; master_architecture.md, section 3

## Description
Implement deep production chains where raw materials are extracted, processed through multiple stages, and delivered as finished goods. This is the core of industrial gameplay from CS1's Industries DLC.

- Raw materials: grain, timber, crude oil, iron ore
- Processing: grain->flour->bread, timber->lumber->furniture, oil->petroleum->plastics, ore->steel->machinery
- Storage buildings: warehouses buffer goods between stages
- Unique factories: combine 2+ processed goods into luxury products
- Logistics: goods transported by truck between stages (freight traffic)
- Supply chain disruption: missing input halts production, backed-up output reduces output
- Import/export: goods can be imported or exported at outside connections

## Definition of Done
- [ ] 4 production chains with 3 stages each
- [ ] Storage buildings buffer goods
- [ ] Truck logistics between chain stages
- [ ] Supply chain disruption visible
- [ ] Import/export for missing/surplus goods

## Test Plan
- Integration: Build complete forestry chain, verify furniture produced
- Integration: Remove processing building, verify downstream stops

## Pitfalls
- Freight traffic from production chains can overwhelm road network
- Warehouse capacity needs balancing (too small = bottleneck, too large = waste)
- production.rs already has partial implementation

## Relevant Code
- `crates/simulation/src/production.rs` -- commodity chain logic
- `crates/simulation/src/market.rs` -- goods market
- `crates/simulation/src/imports_exports.rs` -- outside trade
- `crates/simulation/src/outside_connections.rs` -- connection points
