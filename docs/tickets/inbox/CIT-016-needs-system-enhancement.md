# CIT-016: Needs System Enhancement (Safety and Hygiene)

**Priority:** T2 (Depth)
**Complexity:** Low-Medium (1 person-week)
**Dependencies:** None
**Source:** social_agent_simulation.md Section 3.1

## Description

Current Needs has 5 dimensions (hunger, energy, social, fun, comfort). Add 2 more: safety (based on crime grid at home location, decays in high-crime areas), and hygiene (requires water utility, decays without). Adjust weights: hunger 0.20, energy 0.20, social 0.10, fun 0.10, comfort 0.15, safety 0.15, hygiene 0.10. Critical needs (below 20) should trigger behavioral overrides: hunger -> desperate food seeking, safety -> emigration.

## Definition of Done

- [ ] `safety` and `hygiene` fields added to Needs component
- [ ] Weights updated to 7-dimensional system
- [ ] Safety decays based on crime level at home cell
- [ ] Hygiene decays without water utility
- [ ] Critical need threshold (below 20) triggers override behaviors
- [ ] `most_critical()` updated for new fields
- [ ] `overall_satisfaction()` updated with new weights
- [ ] Serialization updated for new fields

## Test Plan

- Unit test: overall_satisfaction with all needs at 100 = 1.0
- Unit test: overall_satisfaction with safety at 0, others at 100 < 0.85
- Unit test: most_critical correctly identifies lowest of 7 needs
- Integration test: citizens in high-crime area have low safety need

## Pitfalls

- Adding fields to Needs requires save migration for existing saves
- Critical need overrides must not create infinite loops in state machine

## Relevant Code

- `crates/simulation/src/citizen.rs` (Needs component, lines 227-276)
- `crates/simulation/src/life_simulation.rs` (update_needs)
