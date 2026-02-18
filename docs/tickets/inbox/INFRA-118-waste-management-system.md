# INFRA-118: Solid Waste Management System (Landfill, Recycling, WtE)
**Priority:** T2
**Complexity:** L (3-5 days)
**Dependencies:** INFRA-059
**Source:** infrastructure_engineering.md, Section 6

## Description
Implement comprehensive waste management: landfills that fill up over time and must be closed (with ongoing post-closure costs), Material Recovery Facilities (MRFs) for recycling, waste-to-energy plants, transfer stations. Recycling economics fluctuate with commodity markets. Waste hierarchy: reduce > reuse > recycle > compost > energy recovery > landfill. NIMBY effect for landfills and WtE plants (land value/happiness penalty nearby).

## Definition of Done
- [ ] Landfill building with finite capacity, fills over time
- [ ] Landfill closure with post-closure monitoring costs
- [ ] MRF for recycling with commodity price fluctuation
- [ ] Waste-to-energy plant (electricity from waste, NIMBY penalty)
- [ ] Transfer stations for remote landfills
- [ ] NIMBY radius for waste facilities
- [ ] Waste management metrics (diversion rate, per-capita waste)
- [ ] Tests pass

## Test Plan
- Unit: Landfill fills at rate proportional to population; full landfill requires closure
- Unit: MRF revenue fluctuates with commodity prices
- Integration: Growing city needs progressively more waste infrastructure

## Pitfalls
- Landfill capacity in game time vs real time needs tuning
- Methane capture from landfills produces energy revenue
- Current `garbage.rs` exists; extend

## Relevant Code
- `crates/simulation/src/garbage.rs` -- existing garbage system
- `crates/simulation/src/pollution.rs` -- landfill pollution
- `crates/simulation/src/economy.rs` -- waste management costs
