# AUDIO-021: Music Transition Smoothness and Quantization

**Category:** Audio / Music
**Priority:** T4
**Source:** sound_design.md -- Section 3.8

## Summary

All music transitions quantized to musical boundaries (beat or bar). Different transition types use different easing curves: stem fade-in/out (2-4 bars, cubic), section crossfade (4-8 bars, quad), crisis onset (1-2 bars, fast), crisis resolution (8-16 bars, slow), stinger duck (0.5 bars, linear), time-of-day shift (16-32 bars, imperceptibly gradual).

## Acceptance Criteria

- [ ] All transitions quantized to beat/bar boundaries
- [ ] Different easing curves per transition type
- [ ] No audible clicks, pops, or jarring cuts
- [ ] Transitions imperceptible during normal play
