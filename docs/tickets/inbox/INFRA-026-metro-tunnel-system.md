# INFRA-026: Metro Tunnel Construction System
**Priority:** T2
**Complexity:** XL (1-2 weeks)
**Dependencies:** INFRA-023, INFRA-022
**Source:** underground_infrastructure.md, Metro Tunnel System

## Description
Implement metro tunnel construction using Bezier curves (like road segments). Tunnels have construction method (cut-and-cover at Shallow layer, bored TBM at Medium/Deep). Cut-and-cover cheaper ($15-20K/cell) but disrupts surface roads during construction. Bored tunnels more expensive ($20-30K/cell) but no surface disruption. TBM requires launch pit. Track curves must have minimum radius (no sharp turns for trains). Tunnel depth layer affects cost, collision, and rendering.

## Definition of Done
- [ ] `MetroTunnel` struct with Bezier curve routing
- [ ] Cut-and-cover vs bored TBM construction methods
- [ ] Construction cost per cell based on method, depth, and soil type
- [ ] Minimum curve radius enforcement
- [ ] Surface disruption for cut-and-cover construction (temporary road closure)
- [ ] TBM launch pit placement
- [ ] Tests pass

## Test Plan
- Unit: Tunnel cost calculation matches expected values
- Unit: Sharp curve below minimum radius rejected
- Integration: Tunnel placement tool works with underground view

## Pitfalls
- Tunnel-road intersection at cut-and-cover must handle traffic rerouting
- Soil type affects TBM cost (rock >> clay)
- Must integrate with `UndergroundOccupancy` for collision

## Relevant Code
- `crates/simulation/src/road_segments.rs` -- Bezier curve patterns to reuse
- `crates/simulation/src/grid.rs` -- cell-level data
