# GOV-006: Mayor Approval Rating

**Priority:** T3 (Differentiation)
**Complexity:** Low (0.5 person-weeks)
**Dependencies:** GOV-001 (factions), GOV-003 (civic trust)
**Source:** social_agent_simulation.md Section 10

## Description

Continuous mayor approval rating (0-100%) displayed prominently. Formula: approval = sum(faction_satisfaction * faction_support_weight). Modified by: recent events (+/-), civic trust, economic conditions. Approval affects: immigration rate (+10% per 10% above 50%), loan interest rates (lower with high approval), event outcomes (positive events more likely with high approval). Approval visible as prominent metric in header UI.

## Definition of Done

- [ ] `MayorApproval` resource (0-100)
- [ ] Computed from faction satisfactions weighted by support
- [ ] Modified by civic trust and recent events
- [ ] Affects immigration rate
- [ ] Affects loan interest rates
- [ ] Displayed in header UI
- [ ] Historical tracking (graph over time)

## Test Plan

- Unit test: all factions satisfied = ~80% approval
- Unit test: all factions unsatisfied = ~20% approval
- Unit test: high approval improves immigration rate

## Pitfalls

- Approval must be responsive enough to feel meaningful but stable enough to plan around

## Relevant Code

- GOV-001 infrastructure
- `crates/simulation/src/immigration.rs`
