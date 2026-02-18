# TEST-019: Integration Test: Apply-Deferred Flush Points

## Priority: T1 (Core)
## Effort: Small (0.5 day)
## Source: testing_strategy.md -- Section 3.6: Apply-Deferred

## Description
Verify that `apply_deferred` flush points are correctly placed: PathRequest components inserted by state_machine are visible to process_path_requests in same frame.

## Acceptance Criteria
- [ ] Test verifies flush between state_machine and process_path_requests
- [ ] PathRequest components visible after flush
- [ ] Using full App::update() (handles schedule correctly)
