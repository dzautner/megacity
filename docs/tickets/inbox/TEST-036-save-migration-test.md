# TEST-036: Save Version Migration Tests

## Priority: T1 (Core -- after SAVE-010)
## Effort: Small (1-2 days)
## Source: testing_strategy.md -- Save Testing

## Description
Test each version migration function independently. Create v1 save data, migrate to v2, verify all fields. Create v2, migrate to v3, etc. Test full chain from oldest to newest.

## Acceptance Criteria
- [ ] Each migration function has dedicated test
- [ ] Test v1 -> v2: new fields have correct defaults
- [ ] Test full chain: v1 -> v2 -> ... -> vN
- [ ] Test skipping versions (v1 -> vN directly)
- [ ] Old test saves stored as fixtures
