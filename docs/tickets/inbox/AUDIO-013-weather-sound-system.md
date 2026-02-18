# AUDIO-013: Weather Sound System

**Category:** Audio / Environmental
**Priority:** T4
**Source:** sound_design.md -- Section 4.1

## Summary

Multi-layer weather audio. Rain has 7 layers (distant wash, close drops, on pavement/rooftops/foliage/water, gutters) with surface-aware volume scaling. Thunder during storms (randomized intervals 8-30s, distance-based delay and filtering). Wind varying by weather state (gentle breeze to howling storm). Snow muffling (low-pass filter, volume reduction on all sounds).

## Details

- Rain intensity from Weather resource (0.6 moderate, 1.0 storm)
- Surface composition analysis: road_fraction, building_fraction, vegetation_fraction, water_fraction
- Thunder: distance determines spectrum (close = crack + rumble, far = low rumble only)
- Wind: volume and pitch vary by weather state and season
- Winter muffling: 0.5-0.7 multiplier on ambience/traffic buses, 2000-4000 Hz low-pass

## Dependencies

- AUDIO-001 (audio bus hierarchy)
- Weather system
- Grid (surface analysis)

## Acceptance Criteria

- [ ] Rain layers scale with intensity and surface type
- [ ] Thunder at random intervals during storms
- [ ] Wind character varies by weather and season
- [ ] Winter muffling applied to all sounds
