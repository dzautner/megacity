# UI-002: Service Coverage Detail Panel
**Priority:** T1
**Complexity:** S
**Dependencies:** SERV-001
**Source:** master_architecture.md, section M2

## Description
Show per-service coverage statistics and utilization. Player needs to know which services are over/under capacity and which areas lack coverage.

- Per service type: buildings count, total capacity, current utilization %, coverage area %
- Coverage percentage: what % of residential cells are within service radius
- Utilization: current_usage / capacity (green < 80%, yellow 80-100%, red > 100%)
- Hover on service building shows coverage radius
- Uncovered area highlighted in overlay

## Definition of Done
- [ ] Service statistics panel showing all service types
- [ ] Utilization color-coded
- [ ] Coverage percentage per service
- [ ] Service radius visible on hover

## Test Plan
- Integration: Build hospital, verify utilization and coverage displayed correctly

## Pitfalls
- Coverage calculation is O(cells * services) -- cache result
- Must distinguish between "in radius" and "actually served" (capacity matters)

## Relevant Code
- `crates/ui/src/info_panel.rs` -- service detail panel
- `crates/simulation/src/services.rs` -- service data
- `crates/simulation/src/happiness.rs:ServiceCoverageGrid` -- coverage data
