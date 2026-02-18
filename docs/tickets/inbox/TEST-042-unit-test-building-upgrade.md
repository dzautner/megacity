# TEST-042: Unit Tests for Building Upgrade/Downgrade

## Priority: T1 (Core)
## Effort: Small (0.5 day)
## Source: testing_strategy.md -- Key Invariants Checklist

## Description
Test building upgrade conditions (land value, services, happiness). Verify building level does not exceed zone max_level. Test downgrade conditions.

## Acceptance Criteria
- [ ] Test upgrade when all conditions met
- [ ] Test no upgrade when conditions not met
- [ ] Test building level <= zone max_level
- [ ] Test downgrade when conditions deteriorate
