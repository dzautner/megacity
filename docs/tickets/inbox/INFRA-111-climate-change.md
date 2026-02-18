# INFRA-111: Climate Change Long-Term Progression
**Priority:** T5
**Complexity:** M (1-3 days)
**Dependencies:** INFRA-072, INFRA-080
**Source:** master_architecture.md, M6

## Description
Model long-term climate change effects over 100+ game-years. Sea level rises based on city emissions (cumulative CO2). Rising sea levels flood coastal areas. Temperature increases affect cooling/heating demand and agricultural productivity. More frequent extreme weather events. Renewable energy policy slows climate change. Creates 100-year gameplay arcs with long-term consequences.

## Definition of Done
- [ ] Cumulative CO2 tracking from city emissions
- [ ] Sea level rise proportional to cumulative emissions
- [ ] Coastal flooding as sea level rises
- [ ] Temperature increase affecting energy demand
- [ ] Increased extreme weather frequency
- [ ] Climate change indicator in stats
- [ ] Tests pass

## Test Plan
- Unit: High-emission city sees faster sea level rise
- Unit: Renewable energy reduces emission growth
- Integration: Coastal city gradually loses land to sea level rise over 50+ game-years

## Pitfalls
- 100-year arcs mean most players never see effects; accelerate timeline
- Sea level rise must update terrain (flood cells permanently)
- Climate denialism is politically sensitive; present as scientific fact

## Relevant Code
- `crates/simulation/src/pollution.rs` -- CO2 emissions
- `crates/simulation/src/weather.rs` -- temperature changes
- `crates/simulation/src/terrain.rs` -- sea level modification
