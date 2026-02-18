# POWER-020: Power Grid Overlay (Coverage, Outages)

## Priority: T1 (Core)

## Description
Implement a power grid overlay showing which cells have power service, which are blacked out, and the location of generators and transmission lines. Color coding: green (powered), red (blacked out), dark (no service).

## Current State
- No power overlay exists.
- No visual representation of power infrastructure.

## Definition of Done
- [ ] Power overlay: green=powered, red=blacked out, dark grey=no power service.
- [ ] Generator locations shown as icons on overlay.
- [ ] Power line routes visible when overlay active.
- [ ] Tooltip: shows power source, distance to nearest generator, priority tier.
- [ ] Blackout pulsing animation for affected areas.
- [ ] Toggle-able from overlay selector.

## Test Plan
- [ ] Visual test: powered areas green, unpowered dark.
- [ ] Visual test: blackout areas pulse red.
- [ ] Visual test: generators show as distinct icons.

## Pitfalls
- Requires POWER-011 (power lines) and POWER-016 (blackouts) for full functionality.
- Can show basic coverage even before full power system is implemented.

## Code References
- `crates/rendering/src/overlay.rs`: overlay system
