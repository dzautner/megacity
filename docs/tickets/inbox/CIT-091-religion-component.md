# CIT-091: Religion Component

**Priority:** T3 (Differentiation)
**Complexity:** Low (0.5 person-weeks)
**Dependencies:** CIT-001 (demographics)
**Source:** social_agent_simulation.md Section 1.1

## Description

Religion attribute for citizens affecting value priorities and voting. Abstract religions (Faith A-D, Secular) similar to abstract ethnic groups. Religious citizens desire worship buildings (Cathedral, etc.) in neighborhood. Religious satisfaction from worship building proximity. Religion influences faction alignment (Devout/Conservative affinity). Cathedral and worship buildings provide community bonding.

## Definition of Done

- [ ] `Religion` enum with 5 abstract variants
- [ ] Religion assigned at spawn with distribution
- [ ] Religious satisfaction from worship building proximity
- [ ] Religion influences faction affinity
- [ ] Cathedral/worship building happiness bonus for religious citizens
- [ ] Secular citizens unaffected by worship buildings

## Test Plan

- Unit test: religious citizen near cathedral gets happiness bonus
- Unit test: secular citizen unaffected by cathedral
- Unit test: religion influences faction alignment

## Pitfalls

- Keep religions abstract to avoid real-world controversy
- Cathedral already exists as ServiceType

## Relevant Code

- `crates/simulation/src/citizen.rs`
- `crates/simulation/src/services.rs` (Cathedral)
