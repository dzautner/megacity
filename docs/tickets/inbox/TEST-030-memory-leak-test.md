# TEST-030: Memory Leak Detection Test

## Priority: T2 (Depth)
## Effort: Small (1 day)
## Source: testing_strategy.md -- Section 5.5: Memory Benchmarks

## Description
Test that entity count stays bounded over 1000 ticks. Entity count should not grow more than 3x from initial count. Test grid memory footprint stays under 5MB.

## Acceptance Criteria
- [ ] Run 1000 ticks with 10K citizens
- [ ] Final entity count < 3x initial count
- [ ] Grid memory footprint < 5MB (256x256 with ~40-byte cells)
- [ ] No unbounded Vec/HashMap growth
