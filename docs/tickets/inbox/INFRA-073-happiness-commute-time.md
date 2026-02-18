# INFRA-073: Traffic-Aware Commute Time in Happiness
**Priority:** T1
**Complexity:** M (1-3 days)
**Dependencies:** INFRA-030
**Source:** master_architecture.md, Section 6.4

## Description
Replace the current simple distance-based commute penalty in happiness with actual traffic-aware travel time. Commute time = BPR-adjusted travel time on the path the citizen actually takes. Longer commute = lower happiness (nonlinear: 10 min = no penalty, 30 min = moderate, 60+ min = severe). This creates the key feedback: congestion -> unhappiness -> emigration.

## Definition of Done
- [ ] Commute time computed from BPR-based pathfinding
- [ ] Happiness penalty nonlinear with commute duration
- [ ] Commute time stored per citizen
- [ ] Average commute time displayed in city stats
- [ ] Tests pass

## Test Plan
- Unit: 10-min commute gives no happiness penalty
- Unit: 60-min commute gives significant penalty
- Integration: Building a highway that reduces commutes improves happiness

## Pitfalls
- Computing commute time per citizen is expensive; cache and update periodically
- Citizens who work from home should have zero commute (future feature)
- Mode matters: transit commute may be perceived differently than driving

## Relevant Code
- `crates/simulation/src/happiness.rs` -- commute penalty computation
- `crates/simulation/src/movement.rs` -- actual travel time tracking
