# CIT-006: Life Stage -- Child (0-5) Behavior

**Priority:** T2 (Depth)
**Complexity:** Low (0.5 person-weeks)
**Dependencies:** CIT-004 (household)
**Source:** social_agent_simulation.md Section 2.1

## Description

Children (0-5) stay home, require childcare (kindergarten or parent), and generate demand for childcare services. Child health affected by household conditions. Children should be visible at home or kindergarten. Transition to SchoolAge at age 6 is automatic if elementary school exists.

## Definition of Done

- [ ] Children assigned to kindergarten if available within radius
- [ ] Childcare demand metric tracked
- [ ] Children's health affected by household pollution, nutrition
- [ ] Automatic transition to SchoolAge at age 6
- [ ] Children count toward household size for housing demand
- [ ] No work, no shopping, no leisure activities for children

## Test Plan

- Unit test: child aged 5 stays in Child stage
- Unit test: child aged 6 transitions to SchoolAge
- Integration test: kindergarten enrollment increases with child population

## Pitfalls

- Children without parents (orphans) need special handling
- Kindergarten capacity limits

## Relevant Code

- `crates/simulation/src/citizen.rs` (LifeStage::Child, line 18)
- `crates/simulation/src/movement.rs` (citizen_state_machine)
