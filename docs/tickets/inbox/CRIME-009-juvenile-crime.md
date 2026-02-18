# CRIME-009: Juvenile Crime System

**Priority:** T3 (Differentiation)
**Complexity:** Low (0.5 person-weeks)
**Dependencies:** CRIME-001 (crime types)
**Source:** social_agent_simulation.md Section 8

## Description

Youth (14-25) have higher crime propensity than adults. Juvenile crime (vandalism, petty theft) is separate from adult crime. Youth centers reduce juvenile crime by 30% in radius. School dropout increases juvenile crime probability by 3x. After-school programs (from schools with budget) reduce juvenile crime by 20%. Juvenile justice: no prison, community service or counseling.

## Definition of Done

- [ ] Juvenile crime rate for ages 14-25 (2x adult base)
- [ ] School dropout multiplier (3x)
- [ ] Youth center reduction (-30%)
- [ ] After-school program reduction (-20%)
- [ ] Juvenile justice (counseling, not prison)
- [ ] Juvenile crime stats separate from adult

## Test Plan

- Unit test: juvenile crime rate higher than adult
- Unit test: youth center reduces juvenile crime
- Unit test: dropout increases juvenile crime

## Pitfalls

- Juvenile crime should be minor (property damage, not violent)

## Relevant Code

- `crates/simulation/src/crime.rs`
- `crates/simulation/src/services.rs`
