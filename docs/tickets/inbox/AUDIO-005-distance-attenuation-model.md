# AUDIO-005: Distance Attenuation and Audio LOD

**Category:** Audio / Technical
**Priority:** T4
**Source:** sound_design.md -- Section 2.4, 1.5

## Summary

Implement distance attenuation tied to orbital camera distance. Three zones: Near (<200 units, full detail), Mid (200-1000, blended), Far (>1000, aggregated wash). Audio LOD reduces individual emitters to aggregate sounds at far zoom. Inverse square law with rolloff clamp.

## Details

- Near zone: individual sounds (car honks, citizen voices, specific machinery)
- Mid zone: cluster sounds (traffic hum per road segment, zone ambient blends)
- Far zone: city-wide wash (aggregate hum, simplified mix)
- Attenuation formula: volume = (ref_distance / distance)^rolloff_factor, clamped to min_volume
- High-frequency rolloff at distance (low-pass filter, cutoff decreases with distance)
- Chunk-based audio aggregation for performance

## Dependencies

- AUDIO-001 (audio bus hierarchy)
- Camera system (OrbitCamera distance)

## Acceptance Criteria

- [ ] Three LOD zones with distinct audio detail levels
- [ ] Smooth transitions between LOD zones
- [ ] High-frequency rolloff at distance
- [ ] Performance budget maintained at all zoom levels
