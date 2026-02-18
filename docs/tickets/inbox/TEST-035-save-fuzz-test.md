# TEST-035: Save File Fuzz Testing

## Priority: T2 (Depth)
## Effort: Medium (2-3 days)
## Source: testing_strategy.md -- Save Testing

## Description
Feed random/corrupted bytes to the save decoder. Verify no panics, no undefined behavior. All failures should produce SaveError, not crash.

## Acceptance Criteria
- [ ] Fuzz test with random bytes
- [ ] Fuzz test with valid header + corrupted body
- [ ] Fuzz test with truncated files
- [ ] Fuzz test with oversized files
- [ ] All cases produce error, never panic
