# INFRA-038: Bus Rapid Transit (BRT) Dedicated Lanes
**Priority:** T2
**Complexity:** M (1-3 days)
**Dependencies:** INFRA-037
**Source:** transportation_simulation.md, Section 4.4

## Description
Implement BRT as a road overlay/variant. A boulevard with dedicated bus lane loses 1 general-purpose lane (reducing auto capacity ~33%) but gains a high-capacity transit corridor. BRT tiers: Lite (signal priority + limited stops, 18-22 km/h, 2-3K pphpd, $2-5M/km), Full (dedicated lane + platforms + TSP, 22-28 km/h, 4-6K pphpd, $5-15M/km), Gold (full separation + stations + passing, 25-35 km/h, 8-15K pphpd, $10-30M/km).

## Definition of Done
- [ ] BRT lane designation on road segments
- [ ] Auto capacity reduced on roads with BRT lanes
- [ ] Bus speed increased on BRT lanes (no traffic delay)
- [ ] Signal priority option reduces intersection delay for buses
- [ ] BRT tier classification
- [ ] Level boarding option reduces dwell time
- [ ] Tests pass

## Test Plan
- Unit: Boulevard with BRT has 33% less auto capacity
- Unit: Bus on BRT lane travels at 25km/h vs 15km/h in mixed traffic
- Integration: BRT corridor handles 5000+ pphpd

## Pitfalls
- BRT lanes on narrow roads may leave zero general-purpose lanes
- Signal priority must integrate with intersection delay model (INFRA-033)
- Visual distinction between regular bus route and BRT needed

## Relevant Code
- `crates/simulation/src/road_segments.rs` -- BRT lane designation
- `crates/simulation/src/grid.rs` -- road type or overlay flag
