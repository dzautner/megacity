# WATER-010: Water Pressure Zones and Distribution

## Priority: T3 (Differentiation)

## Description
Implement water pressure zones where higher elevation areas require pumping stations to maintain adequate water pressure. Buildings above a pressure zone's effective elevation lose water service. Pump stations create pressure zones with limited height capacity.

## Current State
- Water service is binary (coverage area of water-related services).
- No pressure concept.
- No elevation-based service limitation.
- No pump station building.

## Definition of Done
- [ ] Base pressure zone serves buildings up to elevation 50 (arbitrary units).
- [ ] Booster pump station: extends pressure zone by +30 elevation, $200K, 1x1.
- [ ] Buildings above pressure zone elevation have reduced water pressure (lower service quality).
- [ ] No pressure = no water service for high-elevation buildings.
- [ ] Pressure zone overlay showing serviced vs unserviced elevation bands.
- [ ] Multiple pump stations can chain (each adds +30 elevation to zone).

## Test Plan
- [ ] Unit test: building at elevation 30 is within base pressure zone.
- [ ] Unit test: building at elevation 60 is NOT within base zone.
- [ ] Integration test: booster pump extends service to hilltop buildings.

## Pitfalls
- Current terrain may be flat (no elevation variation); feature only matters on hilly maps.
- Pump stations require power; power outage causes water service loss on hills.
- Chain of pump stations creates fragile infrastructure.

## Code References
- `crates/simulation/src/grid.rs`: `Cell.elevation`
- Research: `environment_climate.md` section 2.2
