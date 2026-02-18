# CIT-015: Personality Trait Effects on Behavior

**Priority:** T3 (Differentiation)
**Complexity:** Medium (2 person-weeks)
**Dependencies:** None
**Source:** social_agent_simulation.md Section 3.2

## Description

The `Personality` component (ambition, sociability, materialism, resilience) exists but has minimal behavioral effects. Connect each trait to concrete gameplay: ambition affects education completion probability (+20% at 1.0), job promotion probability (+30%), career change probability. Sociability affects social need decay rate (2x at 0.1), leisure preference, family formation. Materialism affects housing quality sensitivity (2x penalty), salary importance in happiness. Resilience affects stress resistance (halves negative happiness penalties at 1.0), emigration threshold.

## Definition of Done

- [ ] Ambition modifies: education advancement prob, promotion prob, career change prob
- [ ] Sociability modifies: social need decay rate, leisure seeking frequency, marriage probability
- [ ] Materialism modifies: housing quality happiness weight, salary satisfaction weight
- [ ] Resilience modifies: negative happiness penalty scaling, emigration threshold
- [ ] All modifier formulas documented in code comments
- [ ] Personality visible in citizen inspection UI

## Test Plan

- Unit test: high ambition citizen promotes 30% more often
- Unit test: low resilience citizen emigrates at higher happiness threshold
- Unit test: high materialism citizen unhappier in low-quality housing
- Integration test: population-level personality variation produces visible behavioral differences

## Pitfalls

- Personality effects must not produce degenerate behavior (all high-ambition = broken economy)
- Ensure personality effects are balanced and none dominates

## Relevant Code

- `crates/simulation/src/citizen.rs` (Personality component, lines 203-219)
- `crates/simulation/src/life_simulation.rs` (evolve_personality)
- `crates/simulation/src/happiness.rs` (update_happiness)
