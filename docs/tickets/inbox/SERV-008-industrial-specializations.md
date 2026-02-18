# SERV-008: Industrial Specializations (Forest/Farming/Oil/Ore)
**Priority:** T2
**Complexity:** L
**Dependencies:** ZONE-015
**Source:** cities_skylines_analysis.md, section 12.3, 14.7

## Description
Implement industrial district specializations. Generic industrial processes imported raw materials. Specialized districts extract local resources (forest, farming, oil, ore) and have unique production chains.

- Forest industry: harvests trees, processes lumber -> furniture/paper
- Agricultural: farms crops on fertile land -> flour/meat -> food products
- Oil industry: extracts oil from deposits -> refinery -> plastics/fuel
- Ore industry: mines ore -> smelter -> steel/glass
- Resource deposits: finite (except farming), shown in resource overlay
- Each specialization has unique buildings, worker requirements, production output
- Smart players transition from extraction to processing before deposits deplete

## Definition of Done
- [ ] 4 industrial specializations available per district
- [ ] Resource extraction from terrain deposits
- [ ] Unique building types per specialization
- [ ] Production chains with multiple processing stages
- [ ] Resource depletion for non-renewable resources

## Test Plan
- Integration: Specialize district as forestry near trees, verify lumber production
- Integration: Oil deposit depletes over time, verify production drops

## Pitfalls
- natural_resources.rs already tracks resource deposits
- specialization.rs already has partial implementation
- Resource depletion timeline needs to be balanced with game length

## Relevant Code
- `crates/simulation/src/specialization.rs` -- expand specialization system
- `crates/simulation/src/natural_resources.rs` -- resource deposit tracking
- `crates/simulation/src/production.rs` -- production chain processing
- `crates/simulation/src/districts.rs` -- district specialization assignment
