# CIT-040: Welfare System Enhancement

**Priority:** T2 (Depth)
**Complexity:** Medium (1-2 person-weeks)
**Dependencies:** CIT-039 (homelessness pipeline)
**Source:** historical_demographics_services.md Section 5.4

## Description

Enhance welfare system with multiple programs: cash assistance (2-5% of pop, $3-8K/year), food assistance (8-15%, $2-4K), housing assistance (2-5%, $8-15K), unemployment insurance (3-8% of labor, $5-15K), disability services (5-10%, $5-20K). Each program costs money but reduces downstream problems (crime, homelessness, health costs). Social safety net generosity is a policy choice with immigration implications (generous welfare attracts immigrants).

## Definition of Done

- [ ] `WelfarePrograms` resource with 5 program types
- [ ] Each program: eligibility criteria, cost per recipient, coverage percentage
- [ ] Cash assistance prevents homelessness for eligible citizens
- [ ] Food assistance prevents hunger need bottoming out
- [ ] Housing assistance reduces rent burden
- [ ] Unemployment insurance provides temporary income during job search
- [ ] Disability services for low-health citizens
- [ ] Total welfare cost tracked in budget
- [ ] Welfare generosity affects immigration (pull factor)

## Test Plan

- Unit test: eligible citizen receives cash assistance
- Unit test: welfare prevents homelessness for recipients
- Unit test: total cost matches recipients * per-capita cost
- Integration test: cutting welfare increases homelessness and crime

## Pitfalls

- Welfare costs can become large fraction of budget; must be balanced
- Must not create welfare dependency trap (citizens should prefer work)

## Relevant Code

- `crates/simulation/src/welfare.rs` (WelfareStats)
- `crates/simulation/src/homelessness.rs`
