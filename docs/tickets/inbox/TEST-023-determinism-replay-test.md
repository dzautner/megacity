# TEST-023: Replay Determinism Test

## Priority: T3 (Differentiation)
## Effort: Medium (3-5 days)
## Source: testing_strategy.md -- Section 4.4: Replay Testing

## Description
Record player inputs each tick. Replay same inputs with same seed. Compare state hashes at each tick. Divergence = non-determinism bug.

## Acceptance Criteria
- [ ] `InputRecording` resource with per-tick PlayerAction log
- [ ] `run_with_inputs()` helper replays recorded inputs
- [ ] `compute_state_hash()` hashes budget, citizen positions/states, traffic
- [ ] Two runs with same seed produce identical hashes at all ticks
- [ ] Hash divergence pinpoints the exact tick of non-determinism
