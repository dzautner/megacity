# FEAT-048: Heatmap Overlay Visualization Improvements

**Category:** Feature / UI
**Priority:** T2
**Source:** game_design_mechanics.md -- Section 5.1

## Summary

Improve existing overlay system with proper color ramps (warm=problem, cool=healthy), smooth interpolation, legends, and side-by-side comparison. Charts: population over time, budget breakdown, happiness by district. CSV export for data. Trend indicators (arrows showing change direction).

## Details

- Semi-transparent overlays that don't obscure city
- Smooth interpolation between data points (not blocky per-cell)
- Color-blind friendly palettes
- Charts: line, bar, pie for historical data
- Export data as CSV
- Before/after comparison slider

## Acceptance Criteria

- [ ] Smooth gradient overlays (not blocky)
- [ ] Color-blind friendly color ramps
- [ ] Historical trend charts for key metrics
- [ ] Legend displayed with overlay
