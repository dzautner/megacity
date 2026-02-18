# HEALTH-009: Pollution Health Effects with Dose-Response

**Priority:** T2 (Depth)
**Complexity:** Medium (1-2 person-weeks)
**Dependencies:** None
**Source:** social_agent_simulation.md Section 9, master_architecture.md Section 1.11

## Description

Link pollution exposure to health outcomes with dose-response curves. Air pollution: chronic exposure (>6 months in high-pollution area) causes respiratory illness, reducing health by up to -30. Water pollution: contamination above threshold triggers acute illness (cholera, see HEALTH-005). Noise pollution: chronic exposure causes stress (+5% mortality, -10 happiness). Soil contamination: proximity to industrial brownfields causes cancer risk (+0.5% mortality for nearby residents).

## Definition of Done

- [ ] Chronic air pollution exposure tracking per citizen
- [ ] Dose-response: health = health - pollution_years * 5 (max -30)
- [ ] Acute water contamination triggers disease events
- [ ] Noise stress: +5% baseline mortality, -10 happiness
- [ ] Soil contamination cancer risk near industrial sites
- [ ] All effects scale with exposure duration and intensity
- [ ] Health effects visible in citizen detail panel

## Test Plan

- Unit test: 6 months in high pollution reduces health by ~15
- Unit test: noise exposure increases mortality rate by 5%
- Integration test: residents near factories have measurably worse health

## Pitfalls

- Tracking per-citizen pollution exposure history requires new persistent data
- Must differentiate acute (immediate) vs chronic (long-term) exposure

## Relevant Code

- `crates/simulation/src/health.rs` (update_health_grid)
- `crates/simulation/src/pollution.rs` (PollutionGrid)
- `crates/simulation/src/noise.rs` (NoisePollutionGrid)
- `crates/simulation/src/water_pollution.rs`
