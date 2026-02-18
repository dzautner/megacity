# MOD-023: Extract Weather/Climate Parameters to Data Files

## Priority: T2 (Depth)
## Effort: Small (1 day)
## Source: modding_architecture.md -- Data-Driven Architecture

## Description
Weather parameters (seasonal temperatures, precipitation rates, storm probability) are hardcoded. Extract to data files. Allows modders to create different climate zones (tropical, arctic, arid).

## Acceptance Criteria
- [ ] `ClimateConfig` struct: seasonal temps, precipitation, storm probability
- [ ] `assets/data/climate.ron` with climate definitions
- [ ] Weather system reads from data file
- [ ] Multiple climate presets (temperate, tropical, arctic)
