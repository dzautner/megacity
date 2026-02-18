# WASTE-002: Waste Composition Model (Materials Breakdown)

## Priority: T2 (Depth)

## Description
Implement waste composition tracking that breaks waste into material categories: paper/cardboard (25%), food waste (22%), yard waste (12%), plastics (13%), metals (9%), glass (4%), wood (6%), textiles (6%), other (3%). Composition determines recyclable fraction, compostable fraction, and energy content for WTE.

## Current State
- No waste composition tracking.
- Waste is a single aggregate number.

## Definition of Done
- [ ] `WasteComposition` struct with percentages for each material category.
- [ ] `recyclable_fraction()` = paper*0.80 + plastics*0.30 + metals*0.95 + glass*0.90 + wood*0.20 + textiles*0.15.
- [ ] `compostable_fraction()` = food*0.95 + yard*0.98 + paper*0.10 + wood*0.30.
- [ ] `energy_content_btu_per_lb()` = weighted sum of per-material BTU values.
- [ ] Average energy content: ~4,500 BTU/lb for mixed MSW.
- [ ] Composition varies slightly by building type (restaurants = more food waste).
- [ ] Composition feeds into recycling (WASTE-004) and WTE (POWER-014) calculations.

## Test Plan
- [ ] Unit test: recyclable_fraction returns ~0.40 for average composition.
- [ ] Unit test: compostable_fraction returns ~0.34 for average composition.
- [ ] Unit test: energy content = ~4,500 BTU/lb for default composition.
- [ ] Integration test: restaurant waste has higher food fraction.

## Pitfalls
- Per-building composition variation adds complexity; could simplify to city-wide average initially.
- Composition affects both recycling and WTE, creating interdependencies.
- BTU values are US-centric; may need metric equivalent for display.

## Code References
- Research: `environment_climate.md` section 6.1.2
