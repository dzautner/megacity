# INFRA-028: Metro Line Operations and Train Scheduling
**Priority:** T2
**Complexity:** L (3-5 days)
**Dependencies:** INFRA-027
**Source:** underground_infrastructure.md, Metro Operations section

## Description
Implement metro line management: define lines as ordered sequences of stations, assign trains, set headway. Trains move along tunnel at configurable speed (40-80 km/h average). Headway determines service frequency and passenger wait times. Track capacity (trains per hour limited by signaling). Revenue from fares ($1-3/ride). Operating costs per station ($2K/month) and per train ($1.5K/month). Ridership from mode choice model.

## Definition of Done
- [ ] `MetroLine` struct with ordered station list, headway, train count
- [ ] Trains move along line with dwell time at stations
- [ ] Headway configurable per line
- [ ] Fare revenue and operating cost tracking
- [ ] Ridership count per station per line
- [ ] Metro info panel showing line performance
- [ ] Tests pass

## Test Plan
- Unit: 12 stations, 15 trains, 3-min headway produces correct revenue at 80K daily riders
- Unit: Operating cost matches per-component pricing

## Pitfalls
- Train scheduling at single-track sections needs conflict resolution
- Transfer between lines at shared stations needs passenger flow modeling
- Must integrate with mode choice (INFRA-043) for realistic ridership

## Relevant Code
- `crates/simulation/src/movement.rs` -- citizen movement integration
- `crates/simulation/src/economy.rs` -- revenue/expense tracking
