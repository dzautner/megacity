# INFRA-130: Telecommunications Infrastructure (Cell Towers, Fiber)
**Priority:** T3
**Complexity:** M (1-3 days)
**Dependencies:** none
**Source:** infrastructure_engineering.md, Section 8

## Description
Implement telecommunications as buildable infrastructure. Cell towers: macro (1-25 mile range, $$$$), small cell (100-1000m, $$). Fiber optic along roads ("dig once" synergy with road construction). Coverage quality affects property values and enables tech/office zones. Digital divide: areas without broadband have economic and happiness penalties. 5G small cells require dense deployment.

## Definition of Done
- [ ] Cell tower building types (macro, small cell)
- [ ] Coverage radius and capacity per tower type
- [ ] Fiber optic network along roads
- [ ] Coverage quality overlay
- [ ] Digital divide penalty (low coverage = reduced property value)
- [ ] Telecom coverage enables tech industry specialization
- [ ] Tests pass

## Test Plan
- Unit: Macro tower covers 2km radius; small cell covers 500m
- Unit: Area without coverage gets land value penalty
- Integration: Fiber deployment along new roads is cheaper than standalone

## Pitfalls
- Telecom is a new infrastructure layer; keep initial implementation simple
- Cell tower NIMBY (visual impact, health concerns)
- Fiber along roads: should auto-deploy or require manual placement?

## Relevant Code
- `crates/simulation/src/services.rs` -- coverage pattern
- `crates/simulation/src/land_value.rs` -- telecom coverage bonus
