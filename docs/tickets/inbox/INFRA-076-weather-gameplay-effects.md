# INFRA-076: Weather Gameplay Effects (Storms, Snow, Rain)
**Priority:** T2
**Complexity:** M (1-3 days)
**Dependencies:** none
**Source:** master_architecture.md, M3

## Description
Add gameplay effects to weather events. Storms: damage buildings (condition reduction), knock down trees, increase fire risk from lightning. Snow: slow traffic by 20-40%, increase heating demand, require snow plowing (road maintenance cost). Rain: increase road accident rate, trigger flooding in flood plains, reduce solar power output. Currently `weather.rs` has seasonal cycles but minimal gameplay impact.

## Definition of Done
- [ ] Storm events damage buildings and trees
- [ ] Snow reduces traffic speed and increases heating costs
- [ ] Rain increases accident rate and flood risk
- [ ] Rain reduces solar power output
- [ ] Weather effects visible in relevant systems (traffic overlay, power stats)
- [ ] Tests pass

## Test Plan
- Unit: Snow event reduces road speed by 20-40%
- Unit: Storm damages buildings in affected area
- Integration: Seasonal weather creates visible gameplay variation

## Pitfalls
- Weather effects compound with other systems (rain + pollution = acid rain?)
- Snow plowing needs road maintenance budget integration
- Too-frequent severe weather frustrates players; balance frequency

## Relevant Code
- `crates/simulation/src/weather.rs` -- weather system
- `crates/simulation/src/traffic.rs` -- speed reduction
- `crates/simulation/src/disasters.rs` -- storm damage
