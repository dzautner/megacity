# TEST-011: Property-Based Tests for Economy Invariants

## Priority: T2 (Depth)
## Effort: Small (1-2 days)
## Source: testing_strategy.md -- Section 2.4: Property-Based Testing

## Description
Use proptest to verify economy invariants hold for arbitrary inputs: treasury is finite (no NaN/Inf), tax rate in [0.0, 1.0], loan balance >= 0.

## Acceptance Criteria
- [ ] `proptest` crate added as dev dependency
- [ ] Test: for any population and tax rate, treasury is finite
- [ ] Test: for any loan parameters, remaining_balance >= 0
- [ ] Test: for any budget inputs, monthly_income >= 0
- [ ] Regression file captures failures
