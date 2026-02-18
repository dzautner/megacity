# TEST-044: Unit Tests for Loan System

## Priority: T1 (Core)
## Effort: Small (0.5 day)
## Source: testing_strategy.md -- Key Invariants Checklist

## Description
Test loan mechanics: interest calculation, payment schedule, remaining balance always >= 0, credit rating changes.

## Acceptance Criteria
- [ ] Test loan interest calculation
- [ ] Test monthly payment deduction
- [ ] Test remaining_balance >= 0 at all times
- [ ] Test loan payoff clears loan
- [ ] Test credit rating affects available loan terms
