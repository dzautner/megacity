# MOD-029: Extract Production Chain Parameters to Data Files

## Priority: T2 (Depth)
## Effort: Small (1-2 days)
## Source: modding_architecture.md -- Data-Driven Architecture

## Description
Production chain parameters (commodity types, input/output ratios, processing time) are hardcoded. Extract to data files.

## Acceptance Criteria
- [ ] `ProductionDef` struct: commodity, inputs, outputs, processing_time
- [ ] `assets/data/production.ron` with all production chain definitions
- [ ] Production system reads from data file
