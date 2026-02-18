# GOV-010: Faction-Aligned Advisors

**Priority:** T3 (Differentiation)
**Complexity:** Low-Medium (1 person-week)
**Dependencies:** GOV-001 (factions)
**Source:** master_architecture.md Section 1.15

## Description

Replace generic advisors with faction-aligned advisors. Each faction has an advisor who gives advice from their faction's perspective. Environmental advisor warns about pollution. Business advisor pushes for tax cuts. Labor advisor demands social programs. Advisors disagree, creating natural tension. Player must weigh conflicting advice. Advisor satisfaction reflects faction satisfaction.

## Definition of Done

- [ ] 6 advisors mapped to 6 factions
- [ ] Each advisor generates advice from faction perspective
- [ ] Advisors give conflicting recommendations on same issue
- [ ] Advisor satisfaction = faction satisfaction
- [ ] Advisor UI showing all 6 with current mood
- [ ] Advice triggers based on city conditions (threshold-based)

## Test Plan

- Unit test: environmental advisor warns when pollution rises
- Unit test: business advisor recommends tax cuts
- Integration test: conflicting advice appears for controversial decisions

## Pitfalls

- Advisor system already exists; extend, don't replace entirely

## Relevant Code

- `crates/simulation/src/advisors.rs` (AdvisorPanel)
