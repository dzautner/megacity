# CIT-058: Building Age and Depreciation

**Priority:** T2 (Depth)
**Complexity:** Low (0.5 person-weeks)
**Dependencies:** None
**Source:** master_architecture.md Section 1.4

## Description

Buildings age over time, losing quality. New field `construction_day: u32` on Building component. Building age = current_day - construction_day. Quality depreciation: 1% per year without maintenance, capped at 50% loss. Maintenance cost prevents depreciation (included in service expenses). Old buildings have: lower land value contribution, lower housing quality, higher fire risk (+10% at 50 years). Renovation mechanic: spend funds to reset depreciation.

## Definition of Done

- [ ] `construction_day` field on Building component
- [ ] Building age calculation
- [ ] Quality depreciation (1% per year, max 50%)
- [ ] Maintenance cost prevents depreciation
- [ ] Old building fire risk increase
- [ ] Old building housing quality penalty
- [ ] Renovation action (player-triggered, costs money)
- [ ] Building age visible in inspection panel

## Test Plan

- Unit test: 20-year-old building at 80% quality (no maintenance)
- Unit test: maintained building stays at 100%
- Unit test: 50-year building has +10% fire risk
- Integration test: old neighborhoods show declining quality

## Pitfalls

- Depreciation must be slow enough that the player can manage it
- Save migration: existing buildings need a reasonable default construction_day

## Relevant Code

- `crates/simulation/src/buildings.rs` (Building component)
- `crates/simulation/src/fire.rs`
