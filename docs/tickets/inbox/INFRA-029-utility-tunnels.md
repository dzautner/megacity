# INFRA-029: Utility Tunnel System (Late-Game Infrastructure)
**Priority:** T3
**Complexity:** L (3-5 days)
**Dependencies:** INFRA-019, INFRA-020, INFRA-023
**Source:** underground_infrastructure.md, Utility Tunnels section

## Description
Implement utility tunnels as premium late-game infrastructure. Tunnel widths: Narrow (3 slots), Standard (5 slots), Wide (8 slots), Mega (12 slots). Each slot holds one utility line (water, sewer, power, telecom, heating, etc.). Cost: $20K/cell construction + $100/cell/month. Adding utilities to existing tunnel costs only $500/cell (vs $3K+ for individual trenching). Empty slots reserved for future expansion. Vehicle access in wider tunnels reduces repair time. Sensor monitoring reduces failure probability.

## Definition of Done
- [ ] `UtilityTunnel` struct with width, slots, condition
- [ ] `TunnelWidth` enum: Narrow, Standard, Wide, Mega
- [ ] `UtilitySlot` with type, capacity, occupied, condition
- [ ] Tunnel placement tool with Bezier routing
- [ ] Slot filling: add utility to existing tunnel at reduced cost
- [ ] Maintenance cost reduction vs individual pipes
- [ ] Tests pass

## Test Plan
- Unit: Narrow tunnel has 3 slots, Wide has 8
- Unit: Adding utility to empty slot costs $500/cell, not $3K
- Integration: Utility tunnel replaces individual pipes for covered utilities

## Pitfalls
- Break-even analysis: tunnel costs 2.3x more upfront but saves on disruption and future expansion
- Sewer in tunnel still needs gravity slope
- Tunnel condition degrades; repair costs depend on vehicle access

## Relevant Code
- `crates/simulation/src/utilities.rs` -- utility network integration
- `crates/simulation/src/road_segments.rs` -- Bezier routing pattern
