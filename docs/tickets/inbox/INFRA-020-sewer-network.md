# INFRA-020: Gravity Sewer Network
**Priority:** T2
**Complexity:** XL (1-2 weeks)
**Dependencies:** INFRA-019, INFRA-012
**Source:** underground_infrastructure.md, Sewer Network section

## Description
Implement gravity-fed sewer network. Sewers flow downhill using terrain slope; trunk sewers must be placed along downhill paths to treatment plants. Distribution sewers auto-extend along roads. If terrain forces uphill flow, a pump station is required (increased cost and maintenance). Sewer capacity scales with pipe diameter. Overloaded sewers cause sewage overflow events (pollution, health hazard). Separate storm sewers for rainwater optional upgrade.

## Definition of Done
- [ ] `SewerNetwork` with gravity-flow validation
- [ ] Trunk sewer placement tool with slope validation
- [ ] Pump stations for uphill sections
- [ ] Sewer capacity tracking per pipe segment
- [ ] Overflow events when capacity exceeded during rain
- [ ] Sewer coverage overlay mode
- [ ] Tests pass

## Test Plan
- Unit: Sewer placed downhill flows correctly; uphill placement rejected without pump
- Unit: Overflow triggers when rain event + capacity exceeded
- Integration: Buildings without sewer connection show warning icon

## Pitfalls
- Gravity flow requires terrain elevation; flat terrain makes sewer design trivial
- Combined vs separate storm sewers add complexity; start with combined
- Pump station failure during power outage should cause backup

## Relevant Code
- `crates/simulation/src/utilities.rs` -- extend with sewer network
- `crates/simulation/src/terrain.rs` -- elevation for gravity flow
- `crates/simulation/src/weather.rs` -- rain events for overflow
