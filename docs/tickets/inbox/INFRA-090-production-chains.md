# INFRA-090: Production Chains (Commodity System)
**Priority:** T3
**Complexity:** L (3-5 days)
**Dependencies:** INFRA-046
**Source:** master_architecture.md, M4

## Description
Deepen commodity/production chains beyond current basic system. Raw materials (ore, oil, timber, agricultural) -> processed goods (steel, fuel, lumber, food) -> consumer goods. Each step requires a building type. Supply chain failures cause shortages. Freight transport connects production stages. Import/export fills gaps in local production. Specialized industrial districts for each chain.

## Definition of Done
- [ ] At least 3 production chains with 2-3 stages each
- [ ] Production building types per chain stage
- [ ] Input/output requirements per building
- [ ] Supply chain tracking (shortage detection)
- [ ] Import/export fills production gaps
- [ ] Production overlay showing supply chain status
- [ ] Tests pass

## Test Plan
- Unit: Steel mill requires iron ore input; no ore = no steel output
- Unit: Import fills ore shortage at higher cost
- Integration: Complete production chain from raw material to consumer goods

## Pitfalls
- Production chains add complexity; keep initial implementation simple (3 chains max)
- Freight transport must work for goods movement between production stages
- Current `production.rs` exists; extend rather than replace

## Relevant Code
- `crates/simulation/src/production.rs` -- existing production system
- `crates/simulation/src/imports_exports.rs` -- trade
- `crates/simulation/src/specialization.rs` -- industrial specialization
