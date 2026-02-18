# SVC-009: Postal Service Coverage

**Priority:** T2 (Depth)
**Complexity:** Low (0.5 person-weeks)
**Dependencies:** None
**Source:** historical_demographics_services.md Section 5.6

## Description

Postal service provides minor happiness and commercial efficiency bonus. Post offices have coverage radius. MailSortingCenter provides wider coverage. Postal coverage boosts commercial zone productivity by 5% and residential happiness by +3. Modern cities require telecom (CellTower/DataCenter) as a complement. Current PostalCoverage grid exists but may not be fully integrated into happiness.

## Definition of Done

- [ ] PostOffice: local coverage radius
- [ ] MailSortingCenter: wider coverage, processes for multiple post offices
- [ ] Postal coverage boosts commercial productivity +5%
- [ ] Postal coverage boosts residential happiness +3
- [ ] Integration with happiness system (already partially done)
- [ ] Postal stats in service panel

## Test Plan

- Unit test: post office covers expected radius
- Unit test: commercial buildings with postal coverage have higher productivity
- Integration test: placing post offices improves district happiness

## Pitfalls

- Low-priority service; avoid over-engineering

## Relevant Code

- `crates/simulation/src/postal.rs` (PostalCoverage, PostalStats)
- `crates/simulation/src/services.rs` (PostOffice, MailSortingCenter)
