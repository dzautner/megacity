# AUDIO-020: Procedural Audio Generation

**Category:** Audio / Technical
**Priority:** T5
**Source:** sound_design.md -- Section 7

## Summary

Generate some ambient sounds procedurally rather than from samples. Traffic hum synthesis (filtered noise + oscillators), rain synthesis (white noise + filtering + random drops), wind synthesis (band-pass filtered noise with LFO modulation), crowd murmur (filtered noise + random vocal fragments). Reduces asset size and enables infinite variation.

## Details

- Traffic hum: pink noise filtered (200-800 Hz) + low-frequency oscillator for engine idle
- Rain: white noise + resonant filter sweeps + random transient generator for drops
- Wind: band-pass noise with slowly modulating center frequency (LFO) + gust events
- Crowd murmur: brown noise (low) + filtered noise bursts (vocal simulation)
- Hybrid approach: procedural for backgrounds, samples for foreground details

## Dependencies

- AUDIO-001 (Kira supports custom audio sources)

## Acceptance Criteria

- [ ] At least one procedural sound type implemented
- [ ] Procedural sounds indistinguishable from samples at normal play
- [ ] CPU cost within budget
- [ ] Infinite variation (no audible loops)
