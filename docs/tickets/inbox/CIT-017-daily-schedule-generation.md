# CIT-017: Daily Schedule Generation from Demographics

**Priority:** T3 (Differentiation)
**Complexity:** Medium-High (3 person-weeks)
**Dependencies:** CIT-001 (demographics), CIT-004 (household)
**Source:** social_agent_simulation.md Section 4.1

## Description

Replace the deterministic state machine (home->work->shop->leisure->home) with schedule-based behavior. Each citizen generates a daily schedule from demographic-specific templates: OfficeWorker (leave 7-8, work 8h, lunch 12-1, return 17-18), RetailWorker (variable shifts 6-22), Student (school 8-15), Retiree (no work, medical/park/social), Child (school or home), Nightshift (sleep day, work 22-6). Schedule selected by occupation and life stage.

## Definition of Done

- [ ] `DailySchedule` component with ordered list of time-slot activities
- [ ] 6+ schedule templates for different occupations
- [ ] Schedule generation runs once per game-day per citizen
- [ ] State machine reads from schedule instead of hardcoded sequence
- [ ] Variation within templates (+/- 30 min random offset)
- [ ] Weekend schedules differ from weekday
- [ ] Leisure/shopping duration varies by personality

## Test Plan

- Unit test: office worker schedule has work block 8:00-17:00
- Unit test: retail worker shift differs from office worker
- Unit test: retiree has no work block
- Integration test: road traffic peaks at 7-9 AM and 5-7 PM
- Visual test: traffic density overlay shows commute peaks

## Pitfalls

- Schedule system must be LOD-aware; Statistical tier uses aggregate schedules
- Generating schedules for 100K citizens each day is expensive; use templates with minor variation

## Relevant Code

- `crates/simulation/src/movement.rs` (citizen_state_machine)
- `crates/simulation/src/time_of_day.rs` (GameClock)
