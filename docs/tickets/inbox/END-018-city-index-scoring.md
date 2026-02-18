# END-018: Multi-Dimensional City Index Scoring

**Category:** Endgame / Scoring
**Priority:** T3
**Source:** endgame_replayability.md -- Scoring and Achievement Systems

## Summary

Composite City Index score: Population (15%), Economic (20%), Quality of Life (25%), Infrastructure (15%), Environmental (15%), Cultural (10%). City earns titles from Settlement to Megalopolis as index crosses thresholds. Replaces simple population tracking with holistic city evaluation.

## Details

- Each dimension 0-100 with detailed sub-metrics
- Population: raw pop (log-scaled), growth rate, retention, diversity
- Economic: GDP per capita, employment, diversity (HHI), budget health, debt ratio
- Quality of Life: happiness, healthcare, education, safety, housing affordability, commute, green space
- Infrastructure: condition, transit coverage, utility reliability, disaster prep
- Environmental: air/water quality, emissions per capita, renewable %, waste diversion
- Cultural: cultural facilities per capita, events, tourism, landmarks, historic preservation
- Milestone titles displayed prominently in UI

## Dependencies

- All simulation systems (metrics)

## Acceptance Criteria

- [ ] City Index calculated from 6 dimensions
- [ ] Milestone titles awarded at thresholds
- [ ] City Index displayed in UI dashboard
- [ ] Historical tracking of index over time
