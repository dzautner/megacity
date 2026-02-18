# INFRA-125: Infrastructure Financing Options (Bonds, TIF, Impact Fees)
**Priority:** T3
**Complexity:** M (1-3 days)
**Dependencies:** INFRA-054
**Source:** infrastructure_engineering.md, Section 10 (Financing)

## Description
Expand financing beyond current loan system. Options: General Obligation bonds (backed by taxes, low interest), Revenue bonds (backed by project revenue, higher interest), Tax Increment Financing (TIF districts capture growth value), Impact fees (new development pays for infrastructure), Special assessments (benefited property owners pay). Each has gameplay tradeoffs. Existing `loans.rs` handles basic loans; extend with bond types.

## Definition of Done
- [ ] Bond issuance: GO bonds and revenue bonds
- [ ] TIF district designation (captures property tax growth)
- [ ] Impact fee policy (per-building construction surcharge)
- [ ] Each financing method with distinct gameplay tradeoffs
- [ ] Bond debt service in budget
- [ ] Tests pass

## Test Plan
- Unit: GO bond issues debt at low interest rate
- Unit: TIF district captures tax revenue growth within its boundaries
- Unit: Impact fees slow growth but fund infrastructure
- Integration: Player uses mix of financing for large infrastructure project

## Pitfalls
- Too many financial instruments overwhelms player; introduce gradually
- TIF diverts tax revenue from general fund; may create fiscal problems
- Bond default should have severe consequences (credit downgrade)
- Existing `loans.rs` already has credit rating; extend

## Relevant Code
- `crates/simulation/src/loans.rs` -- loan/bond system
- `crates/simulation/src/economy.rs` -- financing integration
- `crates/simulation/src/districts.rs` -- TIF district overlay
