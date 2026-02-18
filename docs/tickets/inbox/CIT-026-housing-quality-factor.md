# CIT-026: Happiness Factor -- Housing Quality

**Priority:** T2 (Depth)
**Complexity:** Low (0.5 person-weeks)
**Dependencies:** None
**Source:** social_agent_simulation.md Section 5.2

## Description

Housing quality happiness component: building level (1-5 maps to 0.2-1.0 quality), building age (decreases 1% per year without maintenance), overcrowding penalty (occupants > capacity * 1.2 = -0.3), utilities bonus (power +0.1, water +0.1), neighborhood (land value / 255 * 0.3). Formula: housing_quality = level_quality * age_factor - overcrowding + utilities + neighborhood. Weight in overall happiness: 0.20.

## Definition of Done

- [ ] `compute_housing_quality()` function
- [ ] Building level maps to quality score 0.2-1.0
- [ ] Building age depreciation factor
- [ ] Overcrowding penalty when occupants exceed capacity
- [ ] Utility bonuses (power, water)
- [ ] Neighborhood quality from land value
- [ ] Weight of 0.20 in overall happiness formula
- [ ] Replace existing flat POWER_BONUS/NO_POWER_PENALTY with composite

## Test Plan

- Unit test: level 5 building with all utilities = max quality
- Unit test: overcrowded building with no power = low quality
- Unit test: old building has lower quality than new building at same level

## Pitfalls

- Building age does not currently exist; needs new field on Building component
- Must not double-count power/water bonuses already in happiness

## Relevant Code

- `crates/simulation/src/happiness.rs` (update_happiness, lines 154-172)
- `crates/simulation/src/buildings.rs` (Building component)
