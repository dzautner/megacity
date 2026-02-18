# CIT-028: Happiness Factor -- Service Coverage Quality

**Priority:** T2 (Depth)
**Complexity:** Low-Medium (1 person-week)
**Dependencies:** SVC-001 (hybrid coverage model)
**Source:** social_agent_simulation.md Section 5.2

## Description

Service coverage happiness component: count of covered services / total service types * quality_average. Currently gives flat bonuses per service type (+3 to +8). Replace with quality-weighted coverage: each service has quality 0.0-1.0 based on distance, capacity, funding. service_happiness = weighted_coverage_fraction * quality_factor. Weight in overall happiness: 0.15.

## Definition of Done

- [ ] Service quality computed per cell (distance decay, capacity, funding)
- [ ] Per-citizen service happiness = fraction of services covered * avg quality
- [ ] Different services weighted differently (health > telecom)
- [ ] Service weights: health 0.2, education 0.2, police 0.15, fire 0.1, parks 0.15, entertainment 0.1, transport 0.05, telecom 0.05
- [ ] Weight of 0.15 in overall happiness
- [ ] Replace flat per-service bonuses

## Test Plan

- Unit test: all 8 services at full quality = 1.0 service happiness
- Unit test: health coverage at 50% quality + nothing else < 0.2
- Integration test: adding service buildings improves nearby citizen happiness

## Pitfalls

- Current ServiceCoverageGrid uses bitflags (binary coverage); need quality grid
- Expanding coverage grid to quality values increases memory 8x

## Relevant Code

- `crates/simulation/src/happiness.rs` (HEALTH_COVERAGE_BONUS etc., lines 161-171)
- `crates/simulation/src/happiness.rs` (ServiceCoverageGrid)
