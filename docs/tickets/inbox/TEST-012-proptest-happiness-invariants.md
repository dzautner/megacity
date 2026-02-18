# TEST-012: Property-Based Tests for Happiness Invariants

## Priority: T2 (Depth)
## Effort: Small (1 day)
## Source: testing_strategy.md -- Section 2.4: Property-Based Testing

## Description
Use proptest to verify happiness output always in [0.0, 100.0] for any combination of inputs. Health always in [0.0, 100.0]. Needs values in [0.0, 100.0].

## Acceptance Criteria
- [ ] For any combination of happiness inputs, output in [0.0, 100.0]
- [ ] For any health inputs, output in [0.0, 100.0]
- [ ] For any needs values, each in [0.0, 100.0]
