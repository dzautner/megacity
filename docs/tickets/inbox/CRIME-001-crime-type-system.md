# CRIME-001: Crime Type System

**Priority:** T2 (Depth)
**Complexity:** Medium (2-3 person-weeks)
**Dependencies:** None
**Source:** social_agent_simulation.md Section 8.1

## Description

Replace single `CrimeGrid` u8 with typed crime system. Crime types: PropertyCrime (burglary, theft, vandalism), ViolentCrime (assault, robbery, homicide), WhiteCollarCrime (fraud, embezzlement), DrugCrime (dealing, possession), OrganizedCrime (racketeering, corruption). Each type has different causes, rates, and policing responses. Property crime driven by poverty and opportunity. Violent crime by poverty + substance abuse + low education. White collar by high-income areas with low oversight.

## Definition of Done

- [ ] `CrimeType` enum with 5 variants
- [ ] Per-type crime grid or per-cell crime type distribution
- [ ] Property crime rate formula: base * poverty_factor * opportunity_factor / police_factor
- [ ] Violent crime rate formula: base * poverty * substance * education_inverse / police
- [ ] White collar crime formula: base * income * (1 - oversight) / police
- [ ] Drug crime formula: base * unemployment * poverty / police
- [ ] Organized crime formula: base * corruption * (1 - police_quality)
- [ ] Total crime = weighted sum of all types
- [ ] Crime overlay shows dominant crime type per area (color-coded)

## Test Plan

- Unit test: high poverty area generates more property crime
- Unit test: wealthy area with low oversight generates white collar crime
- Unit test: police coverage reduces all crime types
- Integration test: diverse crime patterns visible across city zones

## Pitfalls

- 5 separate crime grids (5 * 256 * 256 = 327K bytes) may be excessive; use sparse storage
- Crime types interact (organized crime increases drug crime)

## Relevant Code

- `crates/simulation/src/crime.rs` (CrimeGrid, update_crime)
