# CIT-021: Population Pyramid Resource

**Priority:** T1 (Core)
**Complexity:** Low (0.5 person-weeks)
**Dependencies:** None
**Source:** historical_demographics_services.md Section 2.1

## Description

Create a `PopulationPyramid` resource with age cohorts (0-4, 5-9, ... 85-89, 90+) by gender. Updated periodically by scanning citizens. Displayed as a population pyramid chart in the UI. Pyramid shape reveals city dynamics: broad base = growing, narrow base = aging, bulge = baby boom generation aging through.

## Definition of Done

- [ ] `PopulationPyramid` resource with 19 age bands x 2 genders
- [ ] Updated every 100 ticks by aggregating citizen ages
- [ ] UI widget showing pyramid chart
- [ ] Dependency ratio calculated: (0-14 + 65+) / (15-64)
- [ ] Median age statistic
- [ ] Population growth rate (births - deaths + immigration - emigration)

## Test Plan

- Unit test: age 27 female correctly counted in 25-29 band
- Unit test: dependency ratio correct for known population
- Visual test: pyramid renders correctly with asymmetric distribution

## Pitfalls

- Must account for virtual population, not just real entities
- Performance: scanning all citizens is O(n); use sampling for 100K+

## Relevant Code

- `crates/simulation/src/citizen.rs` (CitizenDetails.age, Gender)
- `crates/simulation/src/virtual_population.rs` (VirtualPopulation)
- `crates/ui/src/info_panel.rs` (stats display)
