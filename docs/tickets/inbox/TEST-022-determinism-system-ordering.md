# TEST-022: Explicit System Ordering for All System Groups

## Priority: T2 (Depth)
## Effort: Small (1-2 days)
## Source: testing_strategy.md -- Section 4.2: System Ordering Non-Determinism

## Description
Audit systems in the last add_systems block (weather, crime, health, etc.) that run "after imports_exports" but have no ordering relative to each other. Add explicit ordering or document as order-independent.

## Acceptance Criteria
- [ ] Audit parallel system groups in lib.rs
- [ ] Systems with shared resource writes have explicit ordering
- [ ] Order-independent systems documented as such
- [ ] No ambiguity warnings from Bevy's schedule checker
