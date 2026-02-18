# POLL-027: Soil Contamination (Brownfield) Overlay

## Priority: T2 (Depth)

## Description
Implement a soil contamination overlay showing brownfield sites and contamination levels. Color coding from clean (green) to heavily contaminated (dark brown). Helps players identify sites needing remediation before development.

## Current State
- No soil contamination overlay.
- No visual indication of contaminated land.

## Definition of Done
- [ ] Overlay color: green (clean, <10), yellow (mild, 10-30), orange (moderate, 30-100), red (heavy, 100-300), dark brown (toxic, 300+).
- [ ] Tooltip: contamination level, source history (former industrial site, landfill, etc.).
- [ ] Brownfield marker icon on contaminated cells when overlay is off.
- [ ] Integration with building placement: warning when placing residential on contaminated cell.
- [ ] Remediation progress indicator on cells undergoing cleanup.

## Test Plan
- [ ] Visual test: former industrial site shows brown overlay.
- [ ] Visual test: tooltip shows contamination level correctly.
- [ ] Visual test: placement warning triggers on contaminated cell.

## Pitfalls
- Depends on POLL-013 (soil contamination grid) for data.
- Source history tracking requires knowing what was on the cell previously.
- Placement warning may confuse players who don't understand contamination.

## Code References
- `crates/rendering/src/overlay.rs`: overlay system
- Research: `environment_climate.md` section 1.4
