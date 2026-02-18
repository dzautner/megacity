# CIT-009: Life Stage -- Adult (26-54) Career and Family

**Priority:** T2 (Depth)
**Complexity:** Medium (1 person-week)
**Dependencies:** CIT-001 (demographics)
**Source:** social_agent_simulation.md Section 2.2-2.3

## Description

Adult (26-54) is the primary working and family-building stage. Career progression through EntryLevel/Junior/MidCareer/Senior/Executive stages with promotion probabilities. Peak fertility window (25-34). Divorce probability peaks years 5-8 of marriage. Job loss probability 2-10% based on economic conditions. Career change probability 3% annually for unhappy workers. Moderate relocation probability (12% base).

## Definition of Done

- [ ] Career stage tracking (EntryLevel through Executive)
- [ ] Promotion probability per year based on performance and ambition
- [ ] Career stage salary multipliers (1.0x to 3.5x)
- [ ] Peak fertility for partnered adults (see CIT-012)
- [ ] Divorce probability based on years married
- [ ] Job loss probability linked to economic conditions
- [ ] Career change for unhappy workers
- [ ] Transition to Senior at age 55

## Test Plan

- Unit test: promotion probabilities match table values
- Unit test: salary multiplier correct for each career stage
- Integration test: experienced workers earn more than entry-level
- Integration test: economic downturn increases job loss rate

## Pitfalls

- Career stage must persist across save/load
- Executive promotion should be rare (only top 5-10%)

## Relevant Code

- `crates/simulation/src/citizen.rs` (LifeStage::Adult, line 21)
- `crates/simulation/src/life_simulation.rs` (salary_payment, job_seeking)
- `crates/simulation/src/education_jobs.rs` (job_matching)
