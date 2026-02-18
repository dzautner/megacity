# GOV-005: Policy System with Genuine Tradeoffs

**Priority:** T3 (Differentiation)
**Complexity:** Medium (2-3 person-weeks)
**Dependencies:** GOV-001 (factions)
**Source:** social_agent_simulation.md Section 10, master_architecture.md Section 1.15

## Description

Replace simple toggle policies with nuanced tradeoff policies. Each policy has positive AND negative effects, and faction reactions. Example: "Rent Control" -> reduces rent burden (-20% rent), pleases Labor (+10), angers Business (-15), reduces housing construction (-30% new residential). Example: "Green Energy Mandate" -> reduces pollution, pleases Environmentalists, increases energy cost +20%, angers Business. Policies should create impossible tradeoffs that define the player's governance style.

## Definition of Done

- [ ] Policy struct with: name, description, positive_effects, negative_effects, faction_reactions
- [ ] At least 20 policies across categories (economic, social, environmental, public safety)
- [ ] Each policy affects at least 2 factions (one positive, one negative)
- [ ] Policy effects apply as simulation modifiers
- [ ] Policy costs (implementation budget)
- [ ] Policy UI showing effects and faction reactions before enacting
- [ ] Some policies are mutually exclusive

## Test Plan

- Unit test: rent control reduces rent burden by 20%
- Unit test: rent control reduces housing construction by 30%
- Unit test: faction reactions correctly applied
- Integration test: policy choices create distinct city personality

## Pitfalls

- Balancing 20+ policies with cross-effects is complex; playtest extensively
- Policies must not have a "correct answer" (all should be genuinely tradeoff-laden)

## Relevant Code

- `crates/simulation/src/policies.rs` (Policies, current toggle system)
