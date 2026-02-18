# TEST-043: Unit Tests for Abandonment Logic

## Priority: T1 (Core)
## Effort: Small (0.5 day)
## Source: testing_strategy.md -- Key Invariants Checklist

## Description
Test abandonment: no utilities -> abandon, very low happiness -> abandon, no road access -> abandon. Verify citizens evicted on abandonment.

## Acceptance Criteria
- [ ] Test building abandons when no power and no water
- [ ] Test building abandons when happiness is critically low
- [ ] Test citizens removed from abandoned building
- [ ] Test abandoned building occupants = 0
