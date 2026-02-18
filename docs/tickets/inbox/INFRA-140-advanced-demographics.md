# INFRA-140: Advanced Demographics (Income Class, Occupation Detail)
**Priority:** T3
**Complexity:** M (1-3 days)
**Dependencies:** INFRA-087
**Source:** master_architecture.md, M4

## Description
Expand citizen demographics with income class (low/medium/high), detailed occupation categories, and household composition. Income class affects housing choices, mode choice, happiness sensitivity. Occupation affects job matching and commute patterns. Demographics drive zone demand differentiation (low-income residential vs high-income).

## Definition of Done
- [ ] Income class per citizen (low/medium/high)
- [ ] Income distribution across population
- [ ] Income affects housing choice (rent affordability)
- [ ] Income affects mode choice (low-income more transit-dependent)
- [ ] Occupation categories beyond basic employment
- [ ] Demographic statistics in info panel
- [ ] Tests pass

## Test Plan
- Unit: Low-income citizens prefer affordable housing
- Unit: High-income citizens more likely to drive
- Integration: Income segregation emerges from market dynamics

## Pitfalls
- Income distribution must be realistic (bell curve, not uniform)
- Current citizen struct may need significant extension
- Income mobility: citizens can change class over time (education, job change)

## Relevant Code
- `crates/simulation/src/citizen.rs` -- citizen demographics
- `crates/simulation/src/wealth.rs` -- wealth/income tracking
